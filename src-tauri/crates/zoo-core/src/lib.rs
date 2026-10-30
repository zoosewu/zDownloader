//! zoo-core: the download core of zoo-downloader.
//!
//! No Tauri, no JS runtime. The Tauri app is a thin shell over this crate's interface.
//! Design: docs/architecture/phase-2-download-core.md

pub mod naming;
pub mod part_file;
pub mod rate;
pub mod store;
