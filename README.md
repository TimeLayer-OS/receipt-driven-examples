# receipt-driven examples

**English** · [Русский](README.ru.md)

[![receipt-verified](receipt-verified.svg)](https://github.com/TimeLayer-OS/timelayer-verifier/tree/main/audit/2026-07-14)

Small, copy-me examples of the **receipt-driven** (a.k.a. *logless*) pattern from
[TimeLayer](https://timelayer-os.com).

## Who this is for

- **You want to see the pattern, not read about it**: 120 lines of Rust you can copy into
  your project today. Run `./run.sh` and watch three scenarios — including a perfectly
  valid receipt getting refused because it belongs to a *different* action.
- **You're evaluating TimeLayer** and want the minimum honest demo: no SDK, no framework,
  one binary verifier and one gate function.

## The idea

The usual way to record that something happened is to **write a log line after you
act**. That log grows forever, costs money to keep, and — worst of all — anyone with
access can edit it after the fact. You are trusting the keeper of the log.

Receipt-driven flips it around: you **require a signed receipt *before* you act**, and
keep the receipt instead of a log line.

A receipt is a self-contained pair of files:

```
cert.tlcert      the certificate
bundle.tlbundle  its supporting evidence
```

signed by an **Ed25519 quorum of independent operators** (no single party — including
TimeLayer — can issue one alone). It verifies **offline**, with no network and no key
server, using the open-source verifier:

```bash
timelayer-verifier verify cert.tlcert bundle.tlbundle --expect <sha256-of-your-action>
# -> VALID FINAL    (exit 0)  authentic, complete, and issued FOR THIS ACTION
# -> UNVERIFIABLE   (exit 1)  refuse (invalid, incomplete, or attests something else)
```

Nothing to keep growing, nothing to tamper with, nothing to trust us about — the proof
travels with the action and anyone can re-check it.

## The rule: fail-closed, bound to the action

> No valid receipt **for this exact action**, no action.

A receipt that is valid *in itself* but was issued for a different action authorizes
nothing — otherwise any one valid receipt would unlock everything (receipt transplant).
That's why the gate hashes your action spec and passes the digest to the verifier via
`--expect`. Any doubt — verifier missing, non-zero exit, unexpected output, digest
mismatch — counts as "not valid" and the action does **not** run.

## Run the example

You need the open-source verifier (one download) and Rust.

```bash
./run.sh
```

`run.sh` downloads the verifier for your OS — **pinned to an exact version and sha256**
(an unpinned `latest` binary would mean your security boundary silently changes with every
release) — builds the example, then runs three scenarios:

1. the bundled valid receipt + the action it attests → the action runs;
2. a **tampered** receipt → refused;
3. the same valid receipt + a **different** action → refused (no receipt transplant).

The verification itself is fully offline.

## What's here

| Path | What |
|------|------|
| `rust/src/main.rs` | The template. `receipt_is_valid_for()` is the reusable gate — it binds the receipt to *your* action via `--expect`; replace `do_the_real_work()` with your program's actual effect. Keep the binding: a gate without `--expect` is not a gate. |
| `sample-action/`   | The action spec the sample receipt attests (its sha256 is the receipt's subject). |
| `sample-receipt/`  | A real receipt pair (`cert.tlcert` + `bundle.tlbundle`) bound to `sample-action/action.json`. |
| `run.sh`           | Pins + digest-checks the verifier, runs the valid / tampered / transplant cases. |

## Where receipts come from

These examples *consume and verify* receipts. Receipts are **issued** by the TimeLayer
network (see [TL-Agent](https://github.com/TimeLayer-OS/TL-Agent) for the agent SDK that
requests authorization receipts and emits provenance receipts). The verifier source is at
[timelayer-verifier](https://github.com/TimeLayer-OS/timelayer-verifier) — read it, build
it, and you depend on no binary you didn't compile.

## License

Apache 2.0 — see [`LICENSE`](LICENSE). Copy these examples into your own project freely.
