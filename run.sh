#!/usr/bin/env bash
# Run the receipt-driven example end to end:
#   1. download the open-source verifier for your OS (if not already here),
#   2. build the example,
#   3. run it against the bundled sample receipt (passes),
#   4. run it against a tampered copy (refused — fail-closed).
#
# No network is needed for the verification itself; the download in step 1 is
# only to fetch the verifier binary once.
set -euo pipefail
cd "$(dirname "$0")"

BIN="$(pwd)/timelayer-verifier"   # absolute: the example runs from a subdir
BASE=https://github.com/TimeLayer-OS/timelayer-verifier/releases/latest/download

if [ ! -x "$BIN" ]; then
  case "$(uname -s)-$(uname -m)" in
    Linux-x86_64)   ASSET=timelayer-verifier-linux-amd64 ;;
    Darwin-arm64)   ASSET=timelayer-verifier-macos-arm64 ;;
    Darwin-x86_64)  ASSET=timelayer-verifier-macos-x86_64 ;;
    *) echo "Download the verifier for your OS from $BASE and save it as $BIN"; exit 1 ;;
  esac
  echo "fetching $ASSET ..."
  curl -fsSL "$BASE/$ASSET" -o "$BIN"
  chmod +x "$BIN"
fi

export TL_VERIFIER="$BIN"

echo
echo "== valid receipt =="
( cd rust && cargo run --quiet -- ../sample-receipt )

echo
echo "== tampered receipt (one flipped byte) =="
TMP="$(mktemp -d)"
cp sample-receipt/cert.tlcert sample-receipt/bundle.tlbundle "$TMP/"
# flip a byte in the bundle
printf '\xff' | dd of="$TMP/bundle.tlbundle" bs=1 seek=40 count=1 conv=notrunc 2>/dev/null
( cd rust && cargo run --quiet -- "$TMP" ) || echo "(refused, as expected)"
rm -rf "$TMP"
