// Scheduler command handlers
use crate::commands::download::AppState;
use crate::download::DownloadScheduler;
use crate::models::DownloadTask;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use uuid::Uuid;

// Scheduler state wrapper
pub struct SchedulerState {
    pub scheduler: Arc<Mutex<DownloadScheduler>>,
}

/// Schedule a download to start at a specific time
#[tauri::command]
pub async fn schedule_download(
    task_id: String,
    scheduled_time: String, // ISO 8601 format
    state: State<'_, AppState>,
    scheduler_state: State<'_, SchedulerState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&task_id).map_err(|e| e.to_string())?;

    // Parse the scheduled time
    let scheduled_dt = DateTime::parse_from_rfc3339(&scheduled_time)
        .map_err(|e| format!("Invalid datetime format: {}", e))?
        .with_timezone(&Utc);

    // Get the task from queue
    let task = {
        let queue = state.queue.lock().await;
        queue
            .get_task(&uuid)
            .ok_or_else(|| format!("Task not found: {}", task_id))?
    };

    // Schedule the task
    let scheduler = scheduler_state.scheduler.lock().await;
    scheduler.schedule_task(task, scheduled_dt).await;

    Ok(())
}

/// Cancel a scheduled download
#[tauri::command]
pub async fn cancel_scheduled_download(
    task_id: String,
    scheduler_state: State<'_, SchedulerState>,
) -> Result<bool, String> {
    let uuid = Uuid::parse_str(&task_id).map_err(|e| e.to_string())?;

    let scheduler = scheduler_state.scheduler.lock().await;
    let cancelled = scheduler.cancel_scheduled_task(uuid).await;

    Ok(cancelled)
}

/// Get all scheduled downloads
#[tauri::command]
pub async fn get_scheduled_downloads(
    scheduler_state: State<'_, SchedulerState>,
) -> Result<Vec<DownloadTask>, String> {
    let scheduler = scheduler_state.scheduler.lock().await;
    let tasks = scheduler.get_scheduled_tasks().await;

    Ok(tasks)
}

/// Retry a failed download
#[tauri::command]
pub async fn retry_download(
    task_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&task_id).map_err(|e| e.to_string())?;

    // Get task from queue
    let mut task = {
        let queue = state.queue.lock().await;
        queue
            .get_task(&uuid)
            .ok_or_else(|| format!("Task not found: {}", task_id))?
    };

    // Check if task is in Failed status
    if !matches!(task.status, crate::models::DownloadStatus::Failed) {
        return Err("Task is not in Failed status".to_string());
    }

    // Increment retry count and reset status
    task.retry_count += 1;
    task.last_retry_at = Some(Utc::now());
    task.status = crate::models::DownloadStatus::Pending;
    task.error_message = None;
    task.progress = 0.0;

    // Update task in queue
    {
        let mut queue = state.queue.lock().await;
        queue.update_task(task);
    }

    Ok(())
}
