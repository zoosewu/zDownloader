//! zoo-module: runs site modules.
//!
//! `zoo-module serve` is the host the app starts (see zoo-module-client's Supervisor);
//! `new` / `check` / `test` are for module authors (see cli).
//! Design: docs/architecture/phase-4-site-modules.md

mod budget;
mod cli;
mod cassette;
mod hooks;
mod link;
mod net;
mod ops;
mod runtime;
mod serve;

use std::process::ExitCode;

fn main() -> ExitCode {
    cli::main()
}
