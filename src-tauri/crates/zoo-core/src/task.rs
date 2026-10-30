//! Data that crosses the zoo-core interface: tasks, events, requests, configuration.

use crate::error::CoreError;
use crate::transfer::RetryPolicy;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TaskId(Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for TaskId {
    type Err = CoreError;
    fn from_str(s: &str) -> Result<Self, CoreError> {
        Uuid::parse_str(s).map(Self).map_err(|_| CoreError::Invalid(format!("not a task id: {s}")))
    }
}

/// ```text
/// Scheduled ─time─▶ Queued ─slot─▶ Running ──▶ Completed
///                    ▲  ▲             │
///             resume │  └─auto retry─ Failed
///                  Paused ◀─pause─────┘   (Queued / Scheduled / Failed can pause too)
/// any unfinished ─cancel─▶ Cancelled
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TaskState {
    Scheduled,
    Queued,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl TaskState {
    pub fn is_finished(self) -> bool {
        matches!(self, TaskState::Completed | TaskState::Cancelled)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: TaskId,
    pub url: String,
    /// Sanitized file name the download will be saved as (before conflict renaming).
    pub name: String,
    pub dir: PathBuf,
    /// Where the finished file actually landed (may differ from `dir/name` after renaming).
    pub final_path: Option<PathBuf>,
    pub state: TaskState,
    pub error: Option<CoreError>,
    pub downloaded: u64,
    pub total: Option<u64>,
    /// Bytes per second; zero unless running.
    #[serde(default)]
    pub speed: u64,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub retry_count: u32,
    /// When a failed task will be re-queued automatically.
    pub next_retry_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressTick {
    pub id: TaskId,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub speed: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum Event {
    /// A task was added or changed state.
    Upserted(Task),
    Removed(TaskId),
    /// Counters of running tasks, batched a few times per second.
    Progress(Vec<ProgressTick>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", content = "at", rename_all = "camelCase")]
pub enum StartMode {
    /// Queue now; starts as soon as a slot is free.
    Now,
    /// Add paused; the user starts it later.
    Later,
    At(DateTime<Utc>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddRequest {
    pub url: String,
    pub filename: Option<String>,
    pub dir: PathBuf,
    pub start: StartMode,
}

/// The engine's slice of the app settings. Applied immediately by `update_config`.
#[derive(Debug, Clone)]
pub struct Config {
    /// `None` means unlimited.
    pub max_concurrent: Option<usize>,
    pub connections_per_task: usize,
    pub segment_size: u64,
    /// Bytes per second across all downloads; `None` means unlimited.
    pub speed_limit: Option<u64>,
    pub auto_retry: AutoRetry,
    /// Per-request retries inside a transfer.
    pub transfer_retry: RetryPolicy,
}

#[derive(Debug, Clone, Copy)]
pub struct AutoRetry {
    pub enabled: bool,
    pub max_attempts: u32,
    /// Delay before the first automatic retry; doubles each time.
    pub interval: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_concurrent: Some(3),
            connections_per_task: 8,
            segment_size: 10 * 1024 * 1024,
            speed_limit: None,
            auto_retry: AutoRetry { enabled: true, max_attempts: 3, interval: Duration::from_secs(30) },
            transfer_retry: RetryPolicy::default(),
        }
    }
}
