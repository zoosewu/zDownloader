//! Shell side of the download core: owns the `DownloadManager` and proxy pool, applies settings to them, and relays core events to the frontend and to system
//! notifications.

use crate::models::settings::AppSettings;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::broadcast::error::RecvError;
use zoo_core::connection::{ConnectionSource, Direct};
use zoo_core::proxy_pool::ProxyPool;
use zoo_core::{CoreError, DownloadManager, Event, TaskId, TaskState};
use zoo_core::Resolver;

/// Frontend event carrying a `zoo_core::Event`.
pub const DOWNLOAD_EVENT: &str = "download-event";
/// Frontend event asking for a full `downloads_list` refetch (events were dropped).
pub const RESYNC_EVENT: &str = "downloads-resync";

pub struct Downloads {
    pub manager: DownloadManager,
    pub pool: Arc<ProxyPool>,
    direct: Arc<Direct>,
}

impl Downloads {
    /// Load tasks and proxies from `data_dir` and start the manager on the current runtime.
    /// `modules` resolves share pages (see `site_modules`).
    pub fn start(data_dir: &Path, settings: &AppSettings, modules: Arc<dyn Resolver>) -> Result<Self, CoreError> {
        let pool = Arc::new(ProxyPool::open(data_dir.join("proxies.json"))?);
        let direct = Arc::new(Direct::new()?);
        let manager = DownloadManager::start_with(settings.engine_config(), data_dir, direct.clone(), modules)?;
        let downloads = Self { manager, pool, direct };
        downloads.apply(settings);
        Ok(downloads)
    }

    /// Make the engine follow `settings`; takes effect for running downloads' next requests.
    pub fn apply(&self, settings: &AppSettings) {
        self.manager.update_config(settings.engine_config());
        self.pool.set_fallback_direct(settings.proxy_fallback_direct);
        let source: Arc<dyn ConnectionSource> = if settings.enable_proxy {
            self.pool.clone()
        } else {
            self.direct.clone()
        };
        self.manager.set_connection_source(source);
    }

    pub async fn shutdown(&self) {
        self.manager.shutdown().await;
        self.pool.persist();
    }
}

/// Relay every core event to the frontend; notify on completion and on final failure.
pub fn forward_events(app: AppHandle) {
    let mut rx = app.state::<Downloads>().manager.subscribe();
    tauri::async_runtime::spawn(async move {
        let mut last_state: HashMap<TaskId, TaskState> = HashMap::new();
        loop {
            match rx.recv().await {
                Ok(event) => {
                    if let Event::Upserted(task) = &event {
                        let previous = last_state.insert(task.id, task.state);
                        if previous != Some(task.state) {
                            notify(&app, task).await;
                        }
                    }
                    if let Event::Removed(id) = &event {
                        last_state.remove(id);
                    }
                    if let Err(e) = app.emit(DOWNLOAD_EVENT, &event) {
                        log::warn!("failed to emit {DOWNLOAD_EVENT}: {e}");
                    }
                }
                Err(RecvError::Lagged(missed)) => {
                    log::warn!("download events lagged by {missed}; asking the frontend to resync");
                    let _ = app.emit(RESYNC_EVENT, ());
                }
                Err(RecvError::Closed) => return,
            }
        }
    });
}

async fn notify(app: &AppHandle, task: &zoo_core::Task) {
    let (title, body) = match task.state {
        TaskState::Completed => ("下載完成", task.name.clone()),
        // A failure that will be retried automatically is not final yet.
        TaskState::Failed if task.next_retry_at.is_none() => (
            "下載失敗",
            format!("{}: {}", task.name, task.error.as_ref().map(ToString::to_string).unwrap_or_default()),
        ),
        _ => return,
    };
    let Some(settings) = app.try_state::<crate::commands::settings::SettingsState>() else { return };
    if !settings.settings.lock().await.enable_notifications {
        return;
    }
    if let Err(e) = app.notification().builder().title(title).body(body).show() {
        log::warn!("failed to show notification: {e}");
    }
}
