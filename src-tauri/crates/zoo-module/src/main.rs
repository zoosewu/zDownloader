//! zoo-module: runs site modules.
//!
//! `zoo-module serve` is the host the app starts (see zoo-module-client's Supervisor).
//! Design: docs/architecture/phase-4-site-modules.md

mod budget;
mod link;
mod net;
mod ops;
mod runtime;
mod serve;

use std::process::ExitCode;

use runtime::HostOptions;

const USAGE: &str = "usage: zoo-module serve [--allow-loopback]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["serve", flags @ ..] if flags.iter().all(|f| *f == "--allow-loopback") => {
            serve::run(HostOptions { allow_loopback: !flags.is_empty() });
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}
