// Zoo Downloader - Cross-platform downloader
mod commands;
mod download;
mod http_bridge;
mod models;
mod modules;
pub mod native_messaging;
mod proxy;
mod storage;
mod utils;

use commands::captcha::{
    cancel_captcha, get_active_captcha_count, get_pending_captchas, submit_captcha_solution,
};
use commands::clipboard::{detect_url, get_clipboard_content};
use commands::download::{
    cancel_download, clear_completed_downloads, create_download, create_download_with_path,
    get_download_list, import_batch_downloads, pause_all_downloads, pause_download,
    resume_all_downloads, retry_download_direct, start_download, AppState,
};
use commands::module_management::{
    delete_module_credentials, get_module_credentials_status, save_module_credentials,
};
use commands::proxy::{
    clear_all_proxies, get_active_proxy_count, get_proxy_list, import_proxy_list, remove_proxy,
    reset_failed_proxies,
};
use commands::scheduler::{
    cancel_scheduled_download, get_scheduled_downloads, retry_download, schedule_download,
    SchedulerState,
};
use commands::settings::{
    export_settings, get_settings, import_settings, load_settings_from_file, reset_settings,
    update_settings, SettingsState,
};
use download::{scan_unfinished_downloads, DownloadEngine, DownloadQueue, DownloadScheduler};
use modules::captcha::CaptchaManager;
use modules::url_detector::ModuleRegistry;
use proxy::pool::ProxyPool;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;
use utils::clipboard::start_clipboard_monitor;

// State for URL detector module registry
pub struct ModuleRegistryState {
    pub registry: Arc<Mutex<ModuleRegistry>>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize settings
    let settings = Arc::new(Mutex::new(load_settings_from_file()));
    let settings_state = SettingsState {
        settings: Arc::clone(&settings),
    };

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

    // Initialize scheduler
    let scheduler = Arc::new(Mutex::new(DownloadScheduler::new()));
    let scheduler_state = SchedulerState {
        scheduler: Arc::clone(&scheduler),
    };

    // Initialize URL detector module registry
    let module_registry_state = ModuleRegistryState {
        registry: Arc::new(Mutex::new(ModuleRegistry::new())),
    };

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ))
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("zoo-downloader".to_string()),
                    },
                ))
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .manage(captcha_manager)
        .manage(settings_state)
        .manage(scheduler_state)
        .manage(module_registry_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            create_download,
            create_download_with_path,
            start_download,
            pause_download,
            cancel_download,
            get_download_list,
            import_batch_downloads,
            pause_all_downloads,
            resume_all_downloads,
            clear_completed_downloads,
            retry_download_direct,
            get_clipboard_content,
            detect_url,
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
            get_settings,
            update_settings,
            export_settings,
            import_settings,
            reset_settings,
            schedule_download,
            cancel_scheduled_download,
            get_scheduled_downloads,
            retry_download,
            // Phase 6: Module Management Commands (T108-T111)
            save_module_credentials,
            get_module_credentials_status,
            delete_module_credentials,
        ])
        .setup(|app| {
            // Log initialization complete
            log::info!("========================================");
            log::info!("Zoo Downloader v{} starting...", env!("CARGO_PKG_VERSION"));
            log::info!("Log files are saved to the app's log directory");
            log::info!("You can find logs in:");
            log::info!("  - Windows: %APPDATA%\\zoo-downloader\\logs\\");
            log::info!("  - macOS: ~/Library/Logs/zoo-downloader/");
            log::info!("  - Linux: ~/.local/share/zoo-downloader/logs/");
            log::info!("========================================");

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

            // Start HTTP server for Chrome extension integration (optional)
            // Can be disabled with DISABLE_NATIVE_MESSAGING_SERVER=1 environment variable
            let disable_native_messaging = std::env::var("DISABLE_NATIVE_MESSAGING_SERVER")
                .unwrap_or_default()
                == "1";

            if !disable_native_messaging {
                let app_handle3 = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    match native_messaging::http_server::start_http_server(app_handle3).await {
                        Ok(_) => {
                            log::info!("Native Messaging HTTP server started successfully on port 45677");
                        }
                        Err(e) => {
                            // Log warning but don't panic - app can continue without Native Messaging
                            log::warn!("Native Messaging HTTP server failed to start:");
                            log::warn!("{}", e);
                        }
                    }
                });
            } else {
                log::info!("Native messaging HTTP server disabled (DISABLE_NATIVE_MESSAGING_SERVER=1)");
            }

            // Start HTTP bridge for development (browser-based frontend)
            // Check environment variable to enable this
            let enable_http_bridge = std::env::var("TAURI_HTTP_BRIDGE")
                .unwrap_or_else(|_| "false".to_string())
                .parse::<bool>()
                .unwrap_or(false);

            if enable_http_bridge {
                let app_handle4 = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    println!("[Tauri] HTTP Bridge enabled for development");
                    http_bridge::start_http_bridge(app_handle4, 3030).await;
                });
            } else {
                println!("[Tauri] HTTP Bridge disabled (production mode)");
            }

            // Initialize system tray
            if let Err(e) = utils::tray::create_tray(app.handle()) {
                eprintln!("Failed to create system tray: {}", e);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
