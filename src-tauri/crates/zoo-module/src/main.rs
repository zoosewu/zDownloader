//! zoo-module: runs site modules.
//!
//! `zoo-module serve` is the host the app starts (see zoo-module-client's Supervisor).
//! Design: docs/architecture/phase-4-site-modules.md

mod runtime;
mod serve;

use std::process::ExitCode;

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("serve") => {
            serve::run();
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("usage: zoo-module serve");
            ExitCode::from(2)
        }
    }
}
