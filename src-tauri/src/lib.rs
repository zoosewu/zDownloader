// Zoo Downloader - Cross-platform downloader
mod commands;
mod download;
mod models;
mod modules;
mod proxy;
mod storage;
mod utils;

use commands::captcha::{
    cancel_captcha, get_active_captcha_count, get_pending_captchas, submit_captcha_solution,
};
use commands::download::{
    cancel_download, clear_completed_downloads, create_download, get_download_list,
    import_batch_downloads, pause_all_downloads, pause_download, resume_all_downloads,
    retry_download_direct, start_download, AppState,
};
use commands::proxy::{
    clear_all_proxies, get_active_proxy_count, get_proxy_list, import_proxy_list, remove_proxy,
    reset_failed_proxies,
};
use download::{scan_unfinished_downloads, DownloadEngine, DownloadQueue};
use modules::captcha::CaptchaManager;
use proxy::pool::ProxyPool;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;
use utils::clipboard::start_clipboard_monitor;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize proxy pool
    let proxy_pool = Arc::new(ProxyPool::new());

    // Initialize CAPTCHA manager
    let captcha_manager = Arc::new(std::sync::Mutex::new(CaptchaManager::new()));

    // Initialize app state
    let app_state = AppState {
        queue: Arc::new(Mutex::new(DownloadQueue::new())),
        engine: Arc::new(Mutex::new(DownloadEngine::new())),
        proxy_pool: Arc::clone(&proxy_pool),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .manage(captcha_manager)
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
            retry_download_direct,
            import_proxy_list,
            get_proxy_list,
            remove_proxy,
            clear_all_proxies,
            get_active_proxy_count,
            reset_failed_proxies,
            get_pending_captchas,
            submit_captcha_solution,
            cancel_captcha,
            get_active_captcha_count,
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
                    if let Err(e) =
                        app_handle.emit("unfinished-downloads-detected", &unfinished_tasks)
                    {
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
