// AppSettings model - application settings

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    // General settings
    pub language: String, // "zh-TW" | "en" | "ja"
    pub theme: Theme,
    pub auto_start: bool,

    // Download settings
    pub default_download_dir: PathBuf,
    pub max_concurrent_downloads: Option<usize>, // None = unlimited
    pub segment_size_mb: u64,                    // default 10MB
    pub thread_count: usize,                     // connections per download, default 8
    pub speed_limit_mode: SpeedLimitMode,
    pub speed_limit_kbps: Option<u64>,

    // Proxy settings
    pub enable_proxy: bool,
    /// When every proxy is unusable, connect directly instead of failing.
    #[serde(default = "default_true")]
    pub proxy_fallback_direct: bool,
    pub proxy_verification_interval_hours: u64, // default 24

    // UI settings
    pub enable_clipboard_monitoring: bool,
    pub enable_notifications: bool,
    pub enable_system_tray: bool,

    // Advanced settings
    pub auto_retry_enabled: bool,
    pub max_retry_count: u32,
    pub retry_interval_secs: u64,
    /// User-Agent for downloads that do not set their own; `None` uses zoo-downloader's.
    #[serde(default)]
    pub user_agent: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
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
            proxy_fallback_direct: true,
            proxy_verification_interval_hours: 24,
            enable_clipboard_monitoring: true,
            enable_notifications: true,
            enable_system_tray: true,
            auto_retry_enabled: true,
            max_retry_count: 3,
            retry_interval_secs: 5,
            user_agent: None,
        }
    }
}

impl AppSettings {
    /// The download engine's slice of the settings.
    pub fn engine_config(&self) -> zoo_core::Config {
        const MB: u64 = 1024 * 1024;
        zoo_core::Config {
            max_concurrent: self.max_concurrent_downloads.filter(|&n| n > 0),
            connections_per_task: self.thread_count.max(1),
            segment_size: self.segment_size_mb.max(1) * MB,
            speed_limit: match self.speed_limit_mode {
                SpeedLimitMode::Fixed => self.speed_limit_kbps.filter(|&k| k > 0).map(|k| k * 1024),
                SpeedLimitMode::Unlimited => None,
            },
            auto_retry: zoo_core::AutoRetry {
                enabled: self.auto_retry_enabled,
                max_attempts: self.max_retry_count,
                interval: std::time::Duration::from_secs(self.retry_interval_secs.max(1)),
            },
            transfer_retry: Default::default(),
            user_agent: self.user_agent.clone().filter(|ua| !ua.trim().is_empty()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_settings_files_without_new_fields_still_load() {
        let mut json = serde_json::to_value(AppSettings::default()).unwrap();
        json.as_object_mut().unwrap().remove("proxyFallbackDirect");
        let loaded: AppSettings = serde_json::from_value(json).unwrap();
        assert!(loaded.proxy_fallback_direct);
    }

    #[test]
    fn engine_config_maps_every_engine_field() {
        let s = AppSettings {
            max_concurrent_downloads: Some(2),
            thread_count: 4,
            segment_size_mb: 5,
            speed_limit_mode: SpeedLimitMode::Fixed,
            speed_limit_kbps: Some(100),
            auto_retry_enabled: false,
            max_retry_count: 7,
            retry_interval_secs: 9,
            ..AppSettings::default()
        };
        let c = s.engine_config();
        assert_eq!(c.max_concurrent, Some(2));
        assert_eq!(c.connections_per_task, 4);
        assert_eq!(c.segment_size, 5 * 1024 * 1024);
        assert_eq!(c.speed_limit, Some(100 * 1024));
        assert!(!c.auto_retry.enabled);
        assert_eq!(c.auto_retry.max_attempts, 7);
        assert_eq!(c.auto_retry.interval.as_secs(), 9);

        let unlimited = AppSettings { speed_limit_mode: SpeedLimitMode::Unlimited, ..s }.engine_config();
        assert_eq!(unlimited.speed_limit, None);
        assert_eq!(AppSettings::default().engine_config().max_concurrent, None, "default is unlimited (FR-013)");
    }
}
