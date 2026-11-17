// Zoo Downloader - Tauri shell over zoo-core
mod add_window;
mod browser_bridge;
mod clipboard_watch;
mod commands;
mod downloads;
mod models;
mod site_modules;
mod storage;
mod utils;

#[cfg(test)]
mod bindings;

use commands::clipboard::get_clipboard_content;
use commands::downloads::{
    categories_match, downloads_add, downloads_cancel, downloads_clear_finished,
    downloads_list, downloads_move, downloads_pause, downloads_pause_all, downloads_remove, downloads_resume,
    downloads_restart, downloads_resume_all, downloads_retry, downloads_segments, downloads_set_tags, downloads_update,
};
use commands::site_modules::{
    captcha_answer, captcha_open_window, captcha_pending, modules_add_source, modules_cancel_install, modules_install,
    modules_list, modules_move, modules_plan_install, modules_refresh, modules_remove, modules_remove_source,
    modules_set_auto_start, modules_set_credentials, modules_set_enabled, modules_sources,
};
use commands::proxy::{proxies_clear, proxies_import, proxies_list, proxies_remove, proxies_reset_failed};
use commands::app::app_prepare_update;
use commands::settings::{
    load_settings_from_file, settings_export, settings_get, settings_import, settings_reset,
    settings_update, SettingsState,
};
use add_window::{add_window_import, add_window_open, add_window_take};
use commands::urls::{urls_inspect, urls_list};
use downloads::Downloads;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

/// The app's own window (tauri.conf.json); the only one that may call app commands.
const MAIN_WINDOW: &str = "main";

// Used by the frontend as a "backend is ready" ping.
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Every app command. Keep in sync with src/backend/index.ts (checked by
/// src/__tests__/command-names.test.ts).
fn app_commands(invoke: tauri::ipc::Invoke) -> bool {
    let handle: fn(tauri::ipc::Invoke) -> bool = tauri::generate_handler![
            greet,
            downloads_add,
            downloads_list,
            downloads_pause,
            downloads_resume,
            downloads_cancel,
            downloads_retry,
            downloads_restart,
            downloads_update,
            downloads_set_tags,
            downloads_remove,
            downloads_move,
            downloads_pause_all,
            downloads_resume_all,
            downloads_clear_finished,
            downloads_segments,
            get_clipboard_content,
            urls_inspect,
            urls_list,
            proxies_import,
            proxies_list,
            proxies_remove,
            proxies_clear,
            proxies_reset_failed,
            settings_get,
            settings_update,
            settings_export,
            settings_import,
            settings_reset,
            modules_list,
            modules_sources,
            modules_add_source,
            modules_remove_source,
            modules_refresh,
            modules_plan_install,
            modules_install,
            modules_cancel_install,
            modules_remove,
            modules_set_enabled,
            modules_set_auto_start,
            modules_move,
            modules_set_credentials,
            captcha_pending,
            captcha_answer,
            captcha_open_window,
            categories_match,
            app_prepare_update,
            add_window_open,
            add_window_take,
            add_window_import,
    ];
    handle(invoke)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings = load_settings_from_file();

    let app = tauri::Builder::default()
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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(SettingsState { settings: Arc::new(Mutex::new(settings.clone())) })
        .manage(add_window::Pending::default())
        // App commands answer the main window only. Captcha windows show other sites' pages and
        // must not reach the app (capabilities cover plugin commands, not these).
        .invoke_handler(move |invoke| {
            let label = invoke.message.webview_ref().label().to_owned();
            if label != MAIN_WINDOW && label != add_window::ADD_WINDOW {
                invoke.resolver.reject(commands::error::CommandError::new("forbidden", "not available in this window"));
                return true;
            }
            app_commands(invoke)
        })
        .setup(move |app| {
            log::info!("Zoo Downloader v{} starting", env!("CARGO_PKG_VERSION"));

            let data_dir = storage::paths::get_app_data_dir();
            remove_legacy_temp_dir();
            let modules = site_modules::SiteModules::open(&data_dir)
                .map_err(|e| format!("cannot open the site-module marketplace: {e}"))?;
            let resolver = modules.resolver.clone();
            app.manage(modules);
            site_modules::start(app.handle().clone(), settings.check_module_updates);
            // DownloadManager spawns its actor on the current runtime.
            let downloads = tauri::async_runtime::block_on(async { Downloads::start(&data_dir, &settings, resolver) })
                .map_err(|e| format!("cannot start the download manager: {e}"))?;
            app.manage(downloads);
            downloads::forward_events(app.handle().clone());

            browser_bridge::start(app.handle().clone());
            clipboard_watch::start(app.handle().clone());

            if let Err(e) = utils::tray::create_tray(app.handle()) {
                log::warn!("Failed to create system tray: {e}");
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app, event| {
        if let tauri::RunEvent::Exit = event {
            // Keep partial downloads and their state for the next start.
            let downloads = app.state::<Downloads>();
            tauri::async_runtime::block_on(downloads.shutdown());
        }
    });
}

/// Before Phase 2, segments were downloaded to %TEMP%/zoo-downloader; that data cannot be resumed.
fn remove_legacy_temp_dir() {
    let legacy = std::env::temp_dir().join("zoo-downloader");
    if legacy.is_dir() {
        if let Err(e) = std::fs::remove_dir_all(&legacy) {
            log::warn!("cannot remove legacy temp dir {}: {e}", legacy.display());
        }
    }
}
