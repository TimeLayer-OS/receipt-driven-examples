//! Receipt-driven action — a minimal, copy-me template.
//!
//! The idea: instead of *writing a log line after you act* (a log that grows
//! forever and that anyone can edit later), you *require a signed receipt
//! before you act*. The receipt is a self-contained pair of files
//! (`cert.tlcert` + `bundle.tlbundle`) signed by an Ed25519 quorum of
//! independent operators, and it verifies **offline** with the open-source
//! `timelayer-verifier`.
//!
//! This program is the "constructor": the gate (`receipt_is_valid`) is the
//! reusable part — keep it as-is. Replace `do_the_real_work` with whatever
//! your program actually does. The rule is **fail-closed**: no `VALID FINAL`,
//! no action.
//!
//! Run it:
//!     # get the verifier from
//!     #   https://github.com/TimeLayer-OS/timelayer-verifier/releases/latest
//!     # then:
//!     TL_VERIFIER=./timelayer-verifier \
//!         cargo run -- ../sample-receipt
//!
//! Try it with a tampered receipt and watch it refuse to act.

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const CERT_FILE: &str = "cert.tlcert";
const BUNDLE_FILE: &str = "bundle.tlbundle";

fn main() -> ExitCode {
    // 1. Where is the verifier? Env var, else `timelayer-verifier` on PATH.
    let verifier: PathBuf = env::var_os("TL_VERIFIER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("timelayer-verifier"));

    // 2. Which receipt authorizes this action? (a directory holding the pair)
    let Some(receipt_dir) = env::args().nth(1).map(PathBuf::from) else {
        eprintln!("usage: guarded-action <receipt-dir>");
        eprintln!("  <receipt-dir> must contain {CERT_FILE} and {BUNDLE_FILE}");
        return ExitCode::from(2);
    };
    let cert = receipt_dir.join(CERT_FILE);
    let bundle = receipt_dir.join(BUNDLE_FILE);

    // 3. THE GATE. Verify offline before doing anything.
    if !receipt_is_valid(&verifier, &cert, &bundle) {
        eprintln!("REFUSED: no valid receipt — action not performed (fail-closed)");
        return ExitCode::FAILURE;
    }

    // 4. Authorized. Do the real work here.
    println!("VALID FINAL — receipt accepted");
    do_the_real_work(&receipt_dir);
    ExitCode::SUCCESS
}

/// The reusable gate. Returns true ONLY when the receipt pair verifies as
/// `VALID FINAL`. Anything else — non-zero exit, missing binary, unexpected
/// output — is treated as "not valid". This is the fail-closed contract.
fn receipt_is_valid(verifier: &Path, cert: &Path, bundle: &Path) -> bool {
    match Command::new(verifier)
        .arg("verify")
        .arg(cert)
        .arg(bundle)
        .output()
    {
        Ok(out) => {
            out.status.success()
                && String::from_utf8_lossy(&out.stdout).trim() == "VALID FINAL"
        }
        Err(_) => false, // verifier missing / unrunnable -> refuse
    }
}

/// Replace this with your program's actual effect (write a file, call an API,
/// move money, ship an order). It runs only after the gate passed.
fn do_the_real_work(receipt_dir: &Path) {
    println!("action executed, authorized by receipt at {}", receipt_dir.display());
}
