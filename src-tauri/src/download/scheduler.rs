// Download scheduler - manages scheduled downloads and auto-retry
#![allow(dead_code)]

use crate::models::DownloadTask;
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration as TokioDuration};

/// Scheduler for managing download tasks
pub struct DownloadScheduler {
    tasks: Arc<Mutex<Vec<ScheduledTask>>>,
    running: Arc<Mutex<bool>>,
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
            running: Arc::new(Mutex::new(false)),
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

    /// Start the scheduler loop
    pub async fn start(&self, callback: Arc<dyn Fn(DownloadTask) + Send + Sync>) {
        let mut running = self.running.lock().await;
        if *running {
            return; // Already running
        }
        *running = true;
        drop(running);

        let tasks_clone = Arc::clone(&self.tasks);
        let running_clone = Arc::clone(&self.running);

        tokio::spawn(async move {
            loop {
                // Check if still running
                {
                    let running = running_clone.lock().await;
                    if !*running {
                        break;
                    }
                }

                // Check for tasks ready to start
                let now = Utc::now();
                let mut ready_tasks = Vec::new();

                {
                    let mut tasks = tasks_clone.lock().await;

                    // Find tasks that should start now
                    let i = 0;
                    while i < tasks.len() {
                        if tasks[i].scheduled_time <= now {
                            let scheduled_task = tasks.remove(i);
                            ready_tasks.push(scheduled_task.task);
                        } else {
                            break; // Tasks are sorted, so we can stop here
                        }
                    }
                }

                // Execute ready tasks via callback
                for task in ready_tasks {
                    callback(task);
                }

                // Sleep for 1 second before checking again
                sleep(TokioDuration::from_secs(1)).await;
            }
        });
    }

    /// Stop the scheduler
    pub async fn stop(&self) {
        let mut running = self.running.lock().await;
        *running = false;
    }

    /// Check if scheduler is running
    pub async fn is_running(&self) -> bool {
        let running = self.running.lock().await;
        *running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = DownloadScheduler::new();
        assert!(!scheduler.is_running().await);
    }

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
