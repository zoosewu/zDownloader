// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use zoo_downloader_lib::native_messaging::is_native_messaging_mode;

#[tokio::main]
async fn main() {
    // Check if running as Chrome Native Messaging host
    if is_native_messaging_mode() {
        // Run in native messaging mode (stdin/stdout communication)
        // This requires creating a minimal Tauri app without a window
        run_native_messaging_mode().await;
    } else {
        // Run normal Tauri app with UI
        zoo_downloader_lib::run();
    }
}

async fn run_native_messaging_mode() {
    // For native messaging, we need access to the app's state
    // but we can't use the normal Tauri window
    // This is a limitation - we'll need to refactor to share state better

    // For now, just log that native messaging mode was requested
    eprintln!("Native messaging mode requested but not fully implemented");
    eprintln!("Please ensure the main app is running to handle downloads from Chrome");

    // Exit with error code
    std::process::exit(1);
}
