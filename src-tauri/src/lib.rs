// Zoo Downloader - Cross-platform downloader
mod commands;
mod download;
mod models;
mod modules;
mod proxy;
mod storage;
mod utils;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet
            // Download commands will be added here
            // commands::create_download,
            // commands::start_download,
            // commands::pause_download,
            // commands::cancel_download,
            // commands::get_download_list,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
