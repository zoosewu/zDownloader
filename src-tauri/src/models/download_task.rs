// DownloadTask model - represents a single download task
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: Uuid,
    pub url: String,
    pub filename: String,
    pub filepath: String,
    pub filesize: u64,
    pub status: DownloadStatus,
    pub progress: f64,  // 0.0 - 100.0
    pub current_speed: u64,  // bytes/sec
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
            created_at: Utc::now(),
            completed_at: None,
        }
    }
}
