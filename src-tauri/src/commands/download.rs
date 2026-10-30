// Download command handlers - Tauri commands for download management
use crate::commands::settings::SettingsState;
use crate::download::{DownloadEngine, DownloadQueue};
use crate::models::DownloadTask;
use crate::modules::url_detector::detect_url_with_modules; // T086: Module detection
use crate::modules::url_detector::DetectionMode; // T086
use crate::modules::registry::ModuleRegistry as SiteModuleRegistry; // T086
use crate::proxy::pool::ProxyPool;
use crate::storage::get_downloads_dir;
use zoo_core::naming::filename_for;
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
    // Also reached from the Chrome extension HTTP endpoint: validate like every other ingress.
    url_validator::validate_url_format(&url)?;
    let filename = filename_for(filename.as_deref(), &url);

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
    log::info!("========== create_download_with_path called ==========");
    log::info!("URL: {}", url);
    log::info!("Filename: {:?}", filename);
    log::info!("Download Path: {}", download_path);

    // Validate URL format using url_validator utility
    log::info!("Validating URL format...");
    url_validator::validate_url_format(&url)?;
    log::info!("✓ URL format validation passed");

    // T086: Detect if URL should be handled by a site module
    log::info!("Detecting site module support...");
    let (detection_mode, suggested_filename) = {
        let site_registry = SiteModuleRegistry::new(); // For now, create empty registry
        let detection_result = detect_url_with_modules(&url, &site_registry);
        log::info!(
            "Detection result: mode={:?}, module={}",
            detection_result.detection_mode,
            detection_result.detector_name
        );
        (detection_result.detection_mode, detection_result.suggested_filename)
    }; // Drop site_registry and detection_result here

    // T086: Fall back to the module's suggested filename, then the URL path
    let filename = filename_for(filename.or(suggested_filename).as_deref(), &url);
    log::info!("Final filename: {}", filename);

    // Validate download path
    log::info!("Validating download path...");
    let path = PathBuf::from(&download_path);
    if !path.exists() {
        log::error!("✗ Download path does not exist: {}", download_path);
        return Err(format!("Download path does not exist: {}", download_path));
    }
    if !path.is_dir() {
        log::error!("✗ Download path is not a directory: {}", download_path);
        return Err(format!("Download path is not a directory: {}", download_path));
    }
    log::info!("✓ Download path is valid");

    // Check write permission by attempting to create a test file
    log::info!("Checking write permissions...");
    let test_file = path.join(".zoo_downloader_test");
    if std::fs::write(&test_file, b"test").is_err() {
        log::error!("✗ Download path is not writable: {}", download_path);
        return Err(format!("Download path is not writable: {}", download_path));
    }
    std::fs::remove_file(&test_file).ok();
    log::info!("✓ Write permissions verified");

    // Build full file path
    let filepath = path.join(&filename).to_string_lossy().to_string();
    log::info!("Full file path: {}", filepath);

    // T086: Create task with module information if detected
    log::info!("Creating download task...");
    let task = match detection_mode {
        DetectionMode::ModuleCustom => {
            log::info!("Creating task with module support (site-module)");
            // Module detected - create task with module info
            DownloadTask::new_with_module(
                url.clone(),
                filename,
                filepath,
                Some("site-module".to_string()), // Module ID (generic for now)
                None,                             // Module metadata (will be filled during execution)
                Some(url),                        // Original URL
                None,                             // Wait time (will be determined during execution)
                true,                             // Requires auth (assume true for site modules)
            )
        }
        _ => {
            log::info!("Creating standard download task (no module)");
            // Direct download - use standard constructor
            DownloadTask::new(url, filename, filepath)
        }
    };
    let task_id = task.id;
    log::info!("✓ Task created with ID: {}", task_id);

    // Add to queue
    log::info!("Adding task to queue...");
    let mut queue = state.queue.lock().await;
    queue.add_task(task);
    log::info!("✓ Task added to queue");

    log::info!("========== create_download_with_path completed ==========");
    Ok(task_id.to_string())
}

#[tauri::command]
pub async fn start_download(
    task_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("========== start_download called ==========");
    log::info!("Task ID: {}", task_id);

    log::info!("Parsing task ID as UUID...");
    let uuid = Uuid::parse_str(&task_id).map_err(|e| {
        log::error!("✗ Failed to parse task ID: {}", e);
        e.to_string()
    })?;
    log::info!("✓ Task ID parsed successfully");

    // Get task from queue
    log::info!("Retrieving task from queue...");
    let task = {
        let queue = state.queue.lock().await;
        let task_result = queue.get_task(&uuid);
        if task_result.is_none() {
            log::error!("✗ Task not found in queue");
            return Err("Task not found".to_string());
        }
        log::info!("✓ Task retrieved from queue");
        task_result.unwrap()
    }; // Lock is released here

    log::info!("Task details: url={}, filename={}", task.url, task.filename);

    // Start download in background
    log::info!("Starting download in background...");
    let queue_clone = Arc::clone(&state.queue);
    let engine = Arc::clone(&state.engine);

    tokio::spawn(async move {
        log::info!("Download background task started for task: {}", task.id);
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
    log::info!("========== import_batch_downloads called ==========");
    log::info!("File path: {}", file_path);

    // Parse URL file
    let path = PathBuf::from(&file_path);
    log::info!("Parsing URL file...");

    let urls = parse_url_file(&path).map_err(|e| {
        log::error!("✗ Failed to parse URL file: {}", e);
        e
    })?;

    log::info!("✓ Successfully parsed {} URLs from file", urls.len());

    let mut task_ids = Vec::new();
    let downloads_dir = get_downloads_dir();
    log::info!("Download directory: {}", downloads_dir.display());

    // Ensure download directory exists
    if let Err(e) = std::fs::create_dir_all(&downloads_dir) {
        let error_msg = format!(
            "Failed to create download directory '{}': {}",
            downloads_dir.display(),
            e
        );
        log::error!("✗ {}", error_msg);
        return Err(error_msg);
    }
    log::info!("✓ Download directory is ready");

    // Create tasks for all URLs
    log::info!("Creating tasks for {} URLs...", urls.len());
    let mut queue = state.queue.lock().await;

    for (index, url) in urls.iter().enumerate() {
        log::info!("Processing URL {}/{}: {}", index + 1, urls.len(), url);

        // Extract filename from URL
        let filename = filename_for(None, url);
        let filepath = downloads_dir.join(&filename).to_string_lossy().to_string();

        log::info!("  → Filename: {}", filename);
        log::info!("  → Filepath: {}", filepath);

        // Create task
        let task = DownloadTask::new(url.clone(), filename, filepath);
        let task_id = task.id.to_string();
        log::info!("  → Task ID: {}", task_id);

        task_ids.push(task_id);
        queue.add_task(task);
    }

    log::info!("✓ Successfully created {} tasks", task_ids.len());
    log::info!("========== import_batch_downloads completed ==========");

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
