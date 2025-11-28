// Download command handlers - Tauri commands for download management
use crate::download::{DownloadEngine, DownloadQueue};
use crate::models::DownloadTask;
use crate::storage::get_downloads_dir;
use crate::utils::parser::parse_url_file;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct AppState {
    pub queue: Arc<Mutex<DownloadQueue>>,
    pub engine: Arc<Mutex<DownloadEngine>>,
}

#[tauri::command]
pub async fn create_download(
    url: String,
    filename: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Extract filename from URL if not provided
    let filename =
        filename.unwrap_or_else(|| url.split('/').next_back().unwrap_or("download").to_string());

    // Create download directory
    let downloads_dir = get_downloads_dir();
    let filepath = downloads_dir.join(&filename).to_string_lossy().to_string();

    // Create task
    let task = DownloadTask::new(url, filename, filepath);
    let task_id = task.id;

    // Add to queue
    let mut queue = state.queue.lock().await;
    queue.add_task(task);

    Ok(task_id.to_string())
}

#[tauri::command]
pub async fn start_download(
    task_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&task_id).map_err(|e| e.to_string())?;

    // Get task from queue
    let task = {
        let queue = state.queue.lock().await;
        queue.get_task(&uuid).ok_or("Task not found")?
    }; // Lock is released here

    // Start download in background
    let queue_clone = Arc::clone(&state.queue);
    let engine = Arc::clone(&state.engine);

    tokio::spawn(async move {
        let mut task = task.clone();
        let app_clone = app.clone();
        let app_clone2 = app.clone();

        // Perform download with progress callbacks
        {
            let engine = engine.lock().await;
            let result = engine
                .download_file(&mut task, move |task_ref| {
                    // Emit progress event
                    if let Err(e) = app_clone.emit("progress-updated", task_ref) {
                        eprintln!("Failed to emit progress event: {}", e);
                    }
                })
                .await;

            if let Err(e) = result {
                eprintln!("Download failed: {}", e);
                task.status = crate::models::DownloadStatus::Failed;
                task.error_message = Some(e.to_string());
                // Emit status changed event
                if let Err(e) = app_clone2.emit("status-changed", &task) {
                    eprintln!("Failed to emit status-changed event: {}", e);
                }
            }
        } // All non-Send types dropped here

        // Update task in queue
        let mut queue = queue_clone.lock().await;
        queue.update_task(task);
    });

    Ok(())
}

#[tauri::command]
pub async fn get_download_list(state: State<'_, AppState>) -> Result<Vec<DownloadTask>, String> {
    let queue = state.queue.lock().await;
    Ok(queue.get_all_tasks())
}

#[tauri::command]
pub async fn pause_download(task_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let uuid = Uuid::parse_str(&task_id).map_err(|e| e.to_string())?;
    let mut queue = state.queue.lock().await;

    if let Some(mut task) = queue.get_task(&uuid) {
        task.status = crate::models::DownloadStatus::Paused;
        queue.update_task(task);
        Ok(())
    } else {
        Err("Task not found".to_string())
    }
}

#[tauri::command]
pub async fn cancel_download(task_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let uuid = Uuid::parse_str(&task_id).map_err(|e| e.to_string())?;
    let mut queue = state.queue.lock().await;

    if queue.remove_task(&uuid) {
        Ok(())
    } else {
        Err("Task not found".to_string())
    }
}

#[tauri::command]
pub async fn import_batch_downloads(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    // Parse URL file
    let path = PathBuf::from(&file_path);
    let urls = parse_url_file(&path)?;

    let mut task_ids = Vec::new();
    let downloads_dir = get_downloads_dir();

    // Create tasks for all URLs
    let mut queue = state.queue.lock().await;
    for url in urls {
        // Extract filename from URL
        let filename = url.split('/').next_back().unwrap_or("download").to_string();
        let filepath = downloads_dir.join(&filename).to_string_lossy().to_string();

        // Create task
        let task = DownloadTask::new(url, filename, filepath);
        task_ids.push(task.id.to_string());
        queue.add_task(task);
    }

    Ok(task_ids)
}

#[tauri::command]
pub async fn pause_all_downloads(state: State<'_, AppState>) -> Result<(), String> {
    let mut queue = state.queue.lock().await;
    let tasks = queue.get_all_tasks();

    for mut task in tasks {
        if task.status == crate::models::DownloadStatus::Downloading {
            task.status = crate::models::DownloadStatus::Paused;
            queue.update_task(task);
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn resume_all_downloads(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let tasks = {
        let queue = state.queue.lock().await;
        queue
            .get_all_tasks()
            .into_iter()
            .filter(|t| t.status == crate::models::DownloadStatus::Paused)
            .collect::<Vec<_>>()
    };

    for task in tasks {
        start_download(task.id.to_string(), state.clone(), app.clone()).await?;
    }

    Ok(())
}

#[tauri::command]
pub async fn clear_completed_downloads(state: State<'_, AppState>) -> Result<usize, String> {
    let mut queue = state.queue.lock().await;
    let tasks = queue.get_all_tasks();
    let mut removed_count = 0;

    for task in tasks {
        if task.status == crate::models::DownloadStatus::Completed {
            queue.remove_task(&task.id);
            removed_count += 1;
        }
    }

    Ok(removed_count)
}
