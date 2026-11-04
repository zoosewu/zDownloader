//! zoo-module-client: everything the app needs to run site modules, without linking V8.
//!
//! Modules run in the separate `zoo-module serve` process. `Supervisor` starts it, talks to it
//! over stdio JSON lines (`protocol`), and restarts it when it dies or hangs.
//! Design: docs/architecture/phase-4-site-modules.md

pub mod manifest;
pub mod protocol;
#[cfg(feature = "core")]
pub mod resolver;
pub mod supervisor;
#[cfg(windows)]
mod job;

pub use protocol::{CallEvent, Captcha, Grants, Limits, ModuleError, ModuleRef, Resolved, Secret};
pub use supervisor::{Interaction, NoInteraction};
pub use supervisor::Supervisor;
