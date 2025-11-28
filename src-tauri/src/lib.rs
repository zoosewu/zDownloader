// Zoo Downloader - Cross-platform downloader
mod commands;
mod download;
mod models;
mod modules;
mod proxy;
mod storage;
mod utils;

use commands::download::{
    AppState, create_download, start_download, pause_download,
    cancel_download, get_download_list, import_batch_downloads,
    pause_all_downloads, resume_all_downloads, clear_completed_downloads,
};
use download::{DownloadQueue, DownloadEngine, scan_unfinished_downloads};
use utils::clipboard::start_clipboard_monitor;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{Manager, Emitter};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize app state
    let app_state = AppState {
        queue: Arc::new(Mutex::new(DownloadQueue::new())),
        engine: Arc::new(Mutex::new(DownloadEngine::new())),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            create_download,
            start_download,
            pause_download,
            cancel_download,
            get_download_list,
            import_batch_downloads,
            pause_all_downloads,
            resume_all_downloads,
            clear_completed_downloads,
        ])
        .setup(|app| {
            // Scan for unfinished downloads and restore them
            let app_handle = app.handle().clone();
            let app_state = app.state::<AppState>();
            let queue_clone = Arc::clone(&app_state.queue);

            tauri::async_runtime::spawn(async move {
                // Scan for unfinished downloads
                let unfinished_tasks = scan_unfinished_downloads();

                if !unfinished_tasks.is_empty() {
                    println!("Found {} unfinished download(s)", unfinished_tasks.len());

                    // Add tasks to queue
                    {
                        let mut queue = queue_clone.lock().await;
                        for task in &unfinished_tasks {
                            queue.add_task(task.clone());
                        }
                    }

                    // Emit event to frontend
                    if let Err(e) = app_handle.emit("unfinished-downloads-detected", &unfinished_tasks) {
                        eprintln!("Failed to emit unfinished-downloads-detected event: {}", e);
                    }
                }
            });

            // Start clipboard monitor in background
            let app_handle2 = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                start_clipboard_monitor(app_handle2).await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
