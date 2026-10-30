//! zoo-core: the download core of zoo-downloader.
//!
//! No Tauri, no JS runtime. The Tauri app is a thin shell over this crate's interface:
//! `DownloadManager` plus the data types in `task`.
//! Design: docs/architecture/phase-2-download-core.md

pub mod connection;
pub mod error;
pub mod manager;
pub mod naming;
pub mod part_file;
pub mod rate;
pub mod store;
pub mod task;
pub mod transfer;

pub use error::CoreError;
pub use manager::DownloadManager;
pub use task::{AddRequest, AutoRetry, Config, Event, ProgressTick, StartMode, Task, TaskId, TaskState};
