// Download command handlers - Tauri commands for download management
use crate::commands::settings::SettingsState;
use crate::download::{DownloadEngine, DownloadQueue};
use crate::models::DownloadTask;
use crate::proxy::pool::ProxyPool;
use crate::storage::get_downloads_dir;
use crate::utils::parser::parse_url_file;
use crate::utils::url_validator;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct AppState {
    pub queue: Arc<Mutex<DownloadQueue>>,
    pub engine: Arc<Mutex<DownloadEngine>>,
    pub proxy_pool: Arc<ProxyPool>,
}

#[tauri::command]
pub async fn create_download(
    url: String,
    filename: Option<String>,
    state: State<'_, AppState>,
    settings_state: State<'_, SettingsState>,
) -> Result<String, String> {
    // Extract filename from URL if not provided
    let filename =
        filename.unwrap_or_else(|| url.split('/').next_back().unwrap_or("download").to_string());

    // Get download directory from settings
    let downloads_dir = {
        let settings = settings_state.settings.lock().await;
        settings.default_download_dir.clone()
    };

    // Ensure download directory exists
    if let Err(e) = std::fs::create_dir_all(&downloads_dir) {
        eprintln!("Warning: Failed to create download directory: {}", e);
        // Fall back to hardcoded directory if settings directory fails
        let fallback_dir = get_downloads_dir();
        std::fs::create_dir_all(&fallback_dir)
            .map_err(|e| format!("Failed to create download directory: {}", e))?;
    }

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
pub async fn create_download_with_path(
    url: String,
    filename: Option<String>,
    download_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Validate URL format using url_validator utility
    url_validator::validate_url_format(&url)?;

    // Extract filename from URL if not provided
    let filename =
        filename.unwrap_or_else(|| url.split('/').next_back().unwrap_or("download").to_string());

    // Validate download path
    let path = PathBuf::from(&download_path);
    if !path.exists() {
        return Err(format!("Download path does not exist: {}", download_path));
    }
    if !path.is_dir() {
        return Err(format!("Download path is not a directory: {}", download_path));
    }

    // Check write permission by attempting to create a test file
    let test_file = path.join(".zoo_downloader_test");
    if std::fs::write(&test_file, b"test").is_err() {
        return Err(format!("Download path is not writable: {}", download_path));
    }
    std::fs::remove_file(&test_file).ok();

    // Build full file path
    let filepath = path.join(&filename).to_string_lossy().to_string();

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
                    // Emit progress event with proper payload format
                    #[derive(Clone, serde::Serialize)]
                    struct ProgressPayload {
                        task: DownloadTask,
                    }

                    let payload = ProgressPayload {
                        task: task_ref.clone(),
                    };

                    if let Err(e) = app_clone.emit("progress-updated", &payload) {
                        eprintln!("Failed to emit progress event: {}", e);
                    }
                })
                .await;

            if let Err(e) = result {
                eprintln!("Download failed: {}", e);

                // Check if all proxies failed
                if e.is_all_proxies_failed() {
                    // Set task status to failed but with special flag
                    task.status = crate::models::DownloadStatus::Failed;
                    task.error_message = Some("All proxies failed".to_string());

                    // Emit special event for proxy failure
                    #[derive(Clone, serde::Serialize)]
                    struct ProxyFailurePayload {
                        task_id: String,
                        task: DownloadTask,
                    }

                    let payload = ProxyFailurePayload {
                        task_id: task.id.to_string(),
                        task: task.clone(),
                    };

                    if let Err(e) = app_clone2.emit("all-proxies-failed", &payload) {
                        eprintln!("Failed to emit all-proxies-failed event: {}", e);
                    }
                } else {
                    // Regular download failure
                    task.status = crate::models::DownloadStatus::Failed;
                    task.error_message = Some(e.to_string());

                    // Emit status changed event with proper payload format
                    #[derive(Clone, serde::Serialize)]
                    struct StatusPayload {
                        task: DownloadTask,
                    }

                    let payload = StatusPayload {
                        task: task.clone(),
                    };

                    if let Err(e) = app_clone2.emit("status-changed", &payload) {
                        eprintln!("Failed to emit status-changed event: {}", e);
                    }
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

/// Retry download with direct connection (without proxy)
/// This is called when user chooses to bypass proxies after all proxies failed
#[tauri::command]
pub async fn retry_download_direct(
    task_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&task_id).map_err(|e| e.to_string())?;

    // Get task from queue
    let task = {
        let queue = state.queue.lock().await;
        queue.get_task(&uuid).ok_or("Task not found")?
    };

    // Reset task status
    let mut task_reset = task.clone();
    task_reset.status = crate::models::DownloadStatus::Pending;
    task_reset.error_message = None;
    task_reset.progress = 0.0;

    {
        let mut queue = state.queue.lock().await;
        queue.update_task(task_reset.clone());
    }

    // Create a new engine WITHOUT proxy for this download
    let engine_no_proxy = Arc::new(Mutex::new(DownloadEngine::new()));
    let queue_clone = Arc::clone(&state.queue);

    tokio::spawn(async move {
        let mut task = task_reset.clone();
        let app_clone = app.clone();
        let app_clone2 = app.clone();

        // Perform download with progress callbacks (using engine without proxy)
        {
            let engine = engine_no_proxy.lock().await;
            let result = engine
                .download_file(&mut task, move |task_ref| {
                    // Emit progress event with proper payload format
                    #[derive(Clone, serde::Serialize)]
                    struct ProgressPayload {
                        task: DownloadTask,
                    }

                    let payload = ProgressPayload {
                        task: task_ref.clone(),
                    };

                    if let Err(e) = app_clone.emit("progress-updated", &payload) {
                        eprintln!("Failed to emit progress event: {}", e);
                    }
                })
                .await;

            if let Err(e) = result {
                eprintln!("Direct download failed: {}", e);
                task.status = crate::models::DownloadStatus::Failed;
                task.error_message = Some(format!("Direct connection failed: {}", e));

                // Emit status changed event with proper payload format
                #[derive(Clone, serde::Serialize)]
                struct StatusPayload {
                    task: DownloadTask,
                }

                let payload = StatusPayload {
                    task: task.clone(),
                };

                if let Err(e) = app_clone2.emit("status-changed", &payload) {
                    eprintln!("Failed to emit status-changed event: {}", e);
                }
            }
        }

        // Update task in queue
        let mut queue = queue_clone.lock().await;
        queue.update_task(task);
    });

    Ok(())
}
