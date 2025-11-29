// Cross-platform path handling
#![allow(dead_code)]

use std::path::PathBuf;

pub fn get_app_data_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        // Windows: %APPDATA%\zoo-downloader\
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("zoo-downloader")
    } else if cfg!(target_os = "macos") {
        // macOS: ~/Library/Application Support/zoo-downloader/
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("zoo-downloader")
    } else {
        // Linux: ~/.config/zoo-downloader/
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("zoo-downloader")
    }
}

pub fn get_downloads_dir() -> PathBuf {
    get_app_data_dir().join("downloads")
}

pub fn get_config_file() -> PathBuf {
    get_app_data_dir().join("config.json")
}

pub fn get_history_file() -> PathBuf {
    get_app_data_dir().join("history.json")
}

pub fn get_proxies_cache_file() -> PathBuf {
    get_app_data_dir().join("proxies.txt")
}
