// AppSettings model - application settings
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    // General settings
    pub language: String, // "zh-TW" | "en" | "ja"
    pub theme: Theme,
    pub auto_start: bool,

    // Download settings
    pub default_download_dir: PathBuf,
    pub max_concurrent_downloads: Option<usize>, // None = unlimited
    pub segment_size_mb: u64,                    // default 10MB
    pub thread_count: usize,                     // default 8
    pub speed_limit_mode: SpeedLimitMode,
    pub speed_limit_kbps: Option<u64>,

    // Proxy settings
    pub enable_proxy: bool,
    pub proxy_verification_interval_hours: u64, // default 24

    // UI settings
    pub enable_clipboard_monitoring: bool,
    pub enable_notifications: bool,
    pub enable_system_tray: bool,

    // Advanced settings
    pub auto_retry_enabled: bool,
    pub max_retry_count: u32,
    pub retry_interval_secs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpeedLimitMode {
    Unlimited,
    Fixed,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "zh-TW".to_string(),
            theme: Theme::Dark,
            auto_start: false,
            default_download_dir: dirs::download_dir().unwrap_or_else(|| PathBuf::from(".")),
            max_concurrent_downloads: None,
            segment_size_mb: 10,
            thread_count: 8,
            speed_limit_mode: SpeedLimitMode::Unlimited,
            speed_limit_kbps: None,
            enable_proxy: false,
            proxy_verification_interval_hours: 24,
            enable_clipboard_monitoring: true,
            enable_notifications: true,
            enable_system_tray: true,
            auto_retry_enabled: true,
            max_retry_count: 3,
            retry_interval_secs: 5,
        }
    }
}
