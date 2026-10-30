// DownloadSegment model - represents a segment of a download task
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadSegment {
    pub id: Uuid,
    pub task_id: Uuid,
    pub segment_id: usize, // Sequential segment ID (0, 1, 2, ...)
    pub start_byte: u64,
    pub end_byte: u64,
    pub downloaded_bytes: u64,
    pub status: SegmentStatus,
    pub thread_id: Option<usize>,
    pub proxy_id: Option<Uuid>,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SegmentStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
}

impl DownloadSegment {
    pub fn new(segment_id: usize, start_byte: u64, end_byte: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_id: Uuid::nil(), // Will be set when attached to a task
            segment_id,
            start_byte,
            end_byte,
            downloaded_bytes: 0,
            status: SegmentStatus::Pending,
            thread_id: None,
            proxy_id: None,
            retry_count: 0,
        }
    }

    pub fn with_task_id(mut self, task_id: Uuid) -> Self {
        self.task_id = task_id;
        self
    }

    pub fn size(&self) -> u64 {
        self.end_byte - self.start_byte + 1
    }

}
