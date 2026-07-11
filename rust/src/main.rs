//! Receipt-driven action — a minimal, copy-me template.
//!
//! The idea: instead of *writing a log line after you act* (a log that grows
//! forever and that anyone can edit later), you *require a signed receipt
//! before you act*. The receipt is a self-contained pair of files
//! (`cert.tlcert` + `bundle.tlbundle`) signed by an Ed25519 quorum of
//! independent operators, and it verifies **offline** with the open-source
//! `timelayer-verifier`.
//!
//! The gate binds the receipt to ONE SPECIFIC action: it hashes the action
//! spec file (sha256) and requires the receipt to attest exactly that digest
//! (`--expect`). A receipt that is perfectly valid — but was issued for a
//! *different* action — is refused. Without this binding, any valid receipt
//! would authorize anything, which is not authorization at all.
//!
//! The rule is **fail-closed**: no `VALID FINAL` *for this digest*, no action.
//!
//! Run it:
//!     ./run.sh            # pins + checks the verifier, runs 3 scenarios
//! or by hand:
//!     TL_VERIFIER=./timelayer-verifier \
//!         cargo run -- ../sample-receipt ../sample-action/action.json
//!
//! Try it with a tampered receipt — or with somebody else's valid receipt —
//! and watch it refuse to act.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use sha2::{Digest, Sha256};

const CERT_FILE: &str = "cert.tlcert";
const BUNDLE_FILE: &str = "bundle.tlbundle";

fn main() -> ExitCode {
    // 1. Where is the verifier? Env var, else `timelayer-verifier` on PATH.
    let verifier: PathBuf = env::var_os("TL_VERIFIER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("timelayer-verifier"));

    // 2. Which receipt, and WHICH ACTION is it supposed to authorize?
    let (Some(receipt_dir), Some(action_file)) = (
        env::args().nth(1).map(PathBuf::from),
        env::args().nth(2).map(PathBuf::from),
    ) else {
        eprintln!("usage: guarded-action <receipt-dir> <action-file>");
        eprintln!("  <receipt-dir>  must contain {CERT_FILE} and {BUNDLE_FILE}");
        eprintln!("  <action-file>  the action spec this receipt must attest");
        return ExitCode::from(2);
    };
    let cert = receipt_dir.join(CERT_FILE);
    let bundle = receipt_dir.join(BUNDLE_FILE);

    // 3. The action's digest — what the receipt MUST attest, byte for byte.
    let Ok(action_bytes) = fs::read(&action_file) else {
        eprintln!("REFUSED: cannot read action file {}", action_file.display());
        return ExitCode::FAILURE;
    };
    let expected_digest = hex(&Sha256::digest(&action_bytes));

    // 4. THE GATE. Verify offline, bound to this exact action, before anything.
    if !receipt_is_valid_for(&verifier, &cert, &bundle, &expected_digest) {
        eprintln!("REFUSED: no valid receipt FOR THIS ACTION — not performed (fail-closed)");
        eprintln!("  expected digest: {expected_digest}");
        return ExitCode::FAILURE;
    }

    // 5. Authorized for this action. Do the real work here.
    println!("VALID FINAL for {expected_digest} — receipt accepted");
    do_the_real_work(&action_file);
    ExitCode::SUCCESS
}

/// The reusable gate. Returns true ONLY when the receipt pair verifies as
/// `VALID FINAL` **and** attests exactly `expected_digest` (the `--expect`
/// flag, verifier v2.0.0+). Anything else — non-zero exit, missing binary,
/// unexpected output, digest mismatch, a verifier too old to know `--expect` —
/// is treated as "not valid". This is the fail-closed contract.
///
/// Do NOT drop the `--expect` argument: a receipt that is valid *in itself*
/// but not bound to your action authorizes nothing (receipt transplant).
fn receipt_is_valid_for(
    verifier: &Path,
    cert: &Path,
    bundle: &Path,
    expected_digest: &str,
) -> bool {
    match Command::new(verifier)
        .arg("verify")
        .arg(cert)
        .arg(bundle)
        .arg("--expect")
        .arg(expected_digest)
        .output()
    {
        Ok(out) => {
            out.status.success()
                && String::from_utf8_lossy(&out.stdout).trim() == "VALID FINAL"
        }
        Err(_) => false, // verifier missing / unrunnable -> refuse
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Replace this with your program's actual effect (write a file, call an API,
/// move money, ship an order). It runs only after the gate passed for the
/// exact action spec you pass in.
fn do_the_real_work(action_file: &Path) {
    println!(
        "action executed, authorized by a receipt bound to {}",
        action_file.display()
    );
}
