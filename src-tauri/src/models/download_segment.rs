// DownloadSegment model - represents a segment of a download task
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSegment {
    pub id: Uuid,
    pub task_id: Uuid,
    pub start_byte: u64,
    pub end_byte: u64,
    pub downloaded_bytes: u64,
    pub status: SegmentStatus,
    pub thread_id: Option<usize>,
    pub proxy_id: Option<Uuid>,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
}

impl DownloadSegment {
    pub fn new(task_id: Uuid, start_byte: u64, end_byte: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_id,
            start_byte,
            end_byte,
            downloaded_bytes: 0,
            status: SegmentStatus::Pending,
            thread_id: None,
            proxy_id: None,
            retry_count: 0,
        }
    }
}
