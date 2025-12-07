// DownloadTask model - represents a single download task
use super::DownloadSegment;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadTask {
    pub id: Uuid,
    pub url: String,
    pub filename: String,
    pub filepath: String,
    pub filesize: u64,
    pub status: DownloadStatus,
    pub progress: f64,                  // 0.0 - 100.0
    pub current_speed: u64,             // bytes/sec
    pub segments: Vec<DownloadSegment>, // Download segments for multi-threaded downloads
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    // Scheduling fields
    pub scheduled_start: Option<DateTime<Utc>>, // When to automatically start this download
    // Retry fields
    pub retry_count: u32,                       // Current retry attempt (0 = first attempt)
    pub max_retries: Option<u32>,               // Override max retries from settings (None = use settings)
    pub last_retry_at: Option<DateTime<Utc>>,  // Last retry timestamp

    // Module integration fields (Feature 004 - Phase 4.6 - T085)
    pub module_id: Option<String>,              // Which module processed this URL (e.g., "k2s")
    pub module_metadata: Option<serde_json::Value>, // Additional metadata from module
    pub original_url: Option<String>,           // Original URL before module processing
    pub wait_time: Option<u32>,                 // Required wait time in seconds (for free accounts)
    pub requires_auth: bool,                    // Whether this download required authentication
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DownloadStatus {
    Pending,
    Initializing,
    Downloading,
    Paused,
    Cancelling,
    Cancelled,
    Completed,
    Failed,
    VerifyingIntegrity,
    MergingSegments,
}

impl DownloadTask {
    pub fn new(url: String, filename: String, filepath: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            url,
            filename,
            filepath,
            filesize: 0,
            status: DownloadStatus::Pending,
            progress: 0.0,
            current_speed: 0,
            segments: Vec::new(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
            scheduled_start: None,
            retry_count: 0,
            max_retries: None,
            last_retry_at: None,
            // Module integration fields (T085)
            module_id: None,
            module_metadata: None,
            original_url: None,
            wait_time: None,
            requires_auth: false,
        }
    }

    /// Create a new download task with module information (T085)
    pub fn new_with_module(
        url: String,
        filename: String,
        filepath: String,
        module_id: Option<String>,
        module_metadata: Option<serde_json::Value>,
        original_url: Option<String>,
        wait_time: Option<u32>,
        requires_auth: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            url,
            filename,
            filepath,
            filesize: 0,
            status: DownloadStatus::Pending,
            progress: 0.0,
            current_speed: 0,
            segments: Vec::new(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
            scheduled_start: None,
            retry_count: 0,
            max_retries: None,
            last_retry_at: None,
            module_id,
            module_metadata,
            original_url,
            wait_time,
            requires_auth,
        }
    }
}
