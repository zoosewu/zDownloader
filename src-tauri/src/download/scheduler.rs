// Download scheduler - manages scheduled downloads and auto-retry

use crate::models::DownloadTask;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Scheduler for managing download tasks
pub struct DownloadScheduler {
    tasks: Arc<Mutex<Vec<ScheduledTask>>>,
}

/// A scheduled download task
#[derive(Debug, Clone)]
struct ScheduledTask {
    task: DownloadTask,
    scheduled_time: DateTime<Utc>,
}

impl DownloadScheduler {
    /// Create a new scheduler
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Schedule a download task to start at a specific time
    pub async fn schedule_task(&self, mut task: DownloadTask, scheduled_time: DateTime<Utc>) {
        task.scheduled_start = Some(scheduled_time);

        let scheduled_task = ScheduledTask {
            task,
            scheduled_time,
        };

        let mut tasks = self.tasks.lock().await;
        tasks.push(scheduled_task);

        // Sort by scheduled time (earliest first)
        tasks.sort_by(|a, b| a.scheduled_time.cmp(&b.scheduled_time));
    }

    /// Cancel a scheduled task
    pub async fn cancel_scheduled_task(&self, task_id: uuid::Uuid) -> bool {
        let mut tasks = self.tasks.lock().await;
        let initial_len = tasks.len();
        tasks.retain(|t| t.task.id != task_id);
        tasks.len() < initial_len
    }

    /// Get all scheduled tasks
    pub async fn get_scheduled_tasks(&self) -> Vec<DownloadTask> {
        let tasks = self.tasks.lock().await;
        tasks.iter().map(|st| st.task.clone()).collect()
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[tokio::test]
    async fn test_schedule_task() {
        let scheduler = DownloadScheduler::new();
        let task = DownloadTask::new(
            "https://example.com/file.zip".to_string(),
            "file.zip".to_string(),
            "/tmp/file.zip".to_string(),
        );

        let scheduled_time = Utc::now() + Duration::hours(1);
        scheduler.schedule_task(task, scheduled_time).await;

        let tasks = scheduler.get_scheduled_tasks().await;
        assert_eq!(tasks.len(), 1);
    }
}
