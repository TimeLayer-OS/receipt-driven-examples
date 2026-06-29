# receipt-driven examples

**English** · [Русский](README.ru.md)

Small, copy-me examples of the **receipt-driven** (a.k.a. *logless*) pattern from
[TimeLayer](https://timelayer-os.com).

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
timelayer-verifier verify cert.tlcert bundle.tlbundle
# -> VALID FINAL    (exit 0)  authentic and complete
# -> UNVERIFIABLE   (exit 1)  refuse
```

Nothing to keep growing, nothing to tamper with, nothing to trust us about — the proof
travels with the action and anyone can re-check it.

## The rule: fail-closed

> No valid receipt, no action.

Any doubt — verifier missing, non-zero exit, unexpected output — counts as "not valid"
and the action does **not** run.

## Run the example

You need the open-source verifier (one download) and Rust.

```bash
./run.sh
```

`run.sh` downloads the verifier for your OS, builds the example, then runs it twice: once
against the bundled **valid** receipt (the action runs) and once against a **tampered**
copy (the action is refused). The verification itself is fully offline.

## What's here

| Path | What |
|------|------|
| `rust/src/main.rs` | The template. `receipt_is_valid()` is the reusable gate — keep it; replace `do_the_real_work()` with your program's actual effect. |
| `sample-receipt/`  | A real receipt pair (`cert.tlcert` + `bundle.tlbundle`) you can verify offline. |
| `run.sh`           | Fetches the verifier and runs the valid + tampered cases. |

## Where receipts come from

These examples *consume and verify* receipts. Receipts are **issued** by the TimeLayer
network (see [TL-Agent](https://github.com/TimeLayer-OS/TL-Agent) for the agent SDK that
requests authorization receipts and emits provenance receipts). The verifier source is at
[timelayer-verifier](https://github.com/TimeLayer-OS/timelayer-verifier) — read it, build
it, and you depend on no binary you didn't compile.

## License

Apache 2.0 — see [`LICENSE`](LICENSE). Copy these examples into your own project freely.
