#!/usr/bin/env bash
# Run the receipt-driven example end to end:
#   1. download the open-source verifier — PINNED version, digest-checked,
#   2. build the example,
#   3. run three scenarios:
#        a) valid receipt + the action it attests   -> runs
#        b) tampered receipt                        -> refused (fail-closed)
#        c) valid receipt + a DIFFERENT action      -> refused (no transplant)
#
# No network is needed for the verification itself; the download in step 1 is
# only to fetch the verifier binary once.
set -euo pipefail
cd "$(dirname "$0")"

# ── supply-chain pin ─────────────────────────────────────────────────────────
# The verifier is pinned to an exact version AND an exact sha256 per platform.
# `releases/latest` is deliberately NOT used: an unpinned binary means your
# security boundary silently changes whenever a new release lands — and a
# compromised release channel becomes a compromised gate.
# TODO(release-signing): once release artifacts ship a publisher signature,
# verify it here BEFORE the digest check.
VERIFIER_VERSION=v2.0.0
BASE="https://github.com/TimeLayer-OS/timelayer-verifier/releases/download/$VERIFIER_VERSION"

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64)
    ASSET=timelayer-verifier-linux-amd64
    SHA256=2739508273ce5537f0f569393f3daff34d2657281c617cacb9353f7a7fb9eb05 ;;
  Darwin-arm64)
    ASSET=timelayer-verifier-macos-arm64
    SHA256=dc734eb76a14736f3663e8dba0593203618e1905bb1fb7df00d285c6d4db32d0 ;;
  Darwin-x86_64)
    ASSET=timelayer-verifier-macos-x86_64
    SHA256=d71e8ee64e842131a836b632d0f425c5a3f0a05a43fbd011a9eb34358d6c0f9b ;;
  *) echo "Unsupported platform: $(uname -s)-$(uname -m)."
     echo "Download $VERIFIER_VERSION for your OS from $BASE, check its digest, save as ./timelayer-verifier"
     exit 1 ;;
esac

BIN="$(pwd)/timelayer-verifier"   # absolute: the example runs from a subdir

digest_of() {  # portable sha256 of a file
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | cut -d' ' -f1
  else shasum -a 256 "$1" | cut -d' ' -f1; fi
}

if [ -x "$BIN" ] && [ "$(digest_of "$BIN")" != "$SHA256" ]; then
  echo "existing ./timelayer-verifier does not match the pinned $VERIFIER_VERSION digest — refusing to use it"
  echo "  expected: $SHA256"
  echo "  actual:   $(digest_of "$BIN")"
  exit 1
fi

if [ ! -x "$BIN" ]; then
  echo "fetching $ASSET ($VERIFIER_VERSION, digest-pinned) ..."
  curl -fsSL "$BASE/$ASSET" -o "$BIN.download"
  ACTUAL="$(digest_of "$BIN.download")"
  if [ "$ACTUAL" != "$SHA256" ]; then
    rm -f "$BIN.download"
    echo "DIGEST MISMATCH — refusing to install the verifier (fail-closed)"
    echo "  expected: $SHA256"
    echo "  actual:   $ACTUAL"
    exit 1
  fi
  mv "$BIN.download" "$BIN"
  chmod +x "$BIN"
fi

export TL_VERIFIER="$BIN"

# ── scenarios ────────────────────────────────────────────────────────────────
echo
echo "== 1. valid receipt + the action it attests =="
( cd rust && cargo run --quiet -- ../sample-receipt ../sample-action/action.json )

echo
echo "== 2. tampered receipt (one flipped byte) =="
TMP="$(mktemp -d)"
cp sample-receipt/cert.tlcert sample-receipt/bundle.tlbundle "$TMP/"
# flip a byte in the bundle
printf '\xff' | dd of="$TMP/bundle.tlbundle" bs=1 seek=40 count=1 conv=notrunc 2>/dev/null
( cd rust && cargo run --quiet -- "$TMP" ../sample-action/action.json ) \
  && { echo "ERROR: tampered receipt was accepted"; exit 1; } \
  || echo "(refused, as expected)"
rm -rf "$TMP"

echo
echo "== 3. valid receipt + a DIFFERENT action (receipt transplant) =="
TMP2="$(mktemp -d)"
sed 's/quarterly numbers, signed off/pay 1000000 to attacker/' sample-action/action.json > "$TMP2/action.json"
( cd rust && cargo run --quiet -- ../sample-receipt "$TMP2/action.json" ) \
  && { echo "ERROR: transplanted receipt was accepted"; exit 1; } \
  || echo "(refused, as expected: the receipt is valid, but not for THIS action)"
rm -rf "$TMP2"

echo
echo "All three scenarios behaved fail-closed. This is the point."
