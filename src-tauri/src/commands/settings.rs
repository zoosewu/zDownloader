//! Settings commands. Every change is saved atomically and applied to the engine at once.

use crate::commands::error::{CmdResult, CommandError};
use crate::downloads::Downloads;
use crate::models::settings::AppSettings;
use crate::storage::paths::get_config_file;
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;
use zoo_core::store::JsonStore;

pub struct SettingsState {
    pub settings: Arc<Mutex<AppSettings>>,
}

fn store() -> JsonStore<AppSettings> {
    JsonStore::new(get_config_file())
}

/// Saved settings, or defaults (written out) if there are none or they cannot be read.
pub fn load_settings_from_file() -> AppSettings {
    match store().load() {
        Ok(Some(settings)) => return settings,
        Ok(None) => {}
        Err(e) => log::error!("cannot read settings, using defaults: {e}"),
    }
    let defaults = AppSettings::default();
    if let Err(e) = store().save(&defaults) {
        log::warn!("cannot save default settings: {e}");
    }
    defaults
}

/// Save, remember and apply `new`.
async fn replace(new: AppSettings, state: &SettingsState, downloads: &Downloads, app: &AppHandle) -> CmdResult<AppSettings> {
    zoo_core::Classifier::new(&new.categories)?;
    store().save(&new)?;
    *state.settings.lock().await = new.clone();
    downloads.apply(&new);
    crate::app_window::apply(app, &new, false);
    Ok(new)
}

#[tauri::command]
pub async fn settings_get(state: State<'_, SettingsState>) -> CmdResult<AppSettings> {
    Ok(state.settings.lock().await.clone())
}

#[tauri::command]
pub async fn settings_update(
    settings: AppSettings,
    state: State<'_, SettingsState>,
    downloads: State<'_, Downloads>,
    app: AppHandle,
) -> CmdResult<AppSettings> {
    replace(settings, &state, &downloads, &app).await
}

#[tauri::command]
pub async fn settings_export(path: String, state: State<'_, SettingsState>) -> CmdResult<()> {
    let settings = state.settings.lock().await.clone();
    JsonStore::new(&path).save(&settings)?;
    Ok(())
}

#[tauri::command]
pub async fn settings_import(
    path: String,
    state: State<'_, SettingsState>,
    downloads: State<'_, Downloads>,
    app: AppHandle,
) -> CmdResult<AppSettings> {
    let imported = JsonStore::<AppSettings>::new(Path::new(&path))
        .load()
        .map_err(|e| CommandError::invalid(format!("not a settings file: {e}")))?
        .ok_or_else(|| CommandError::new("notFound", format!("file not found: {path}")))?;
    replace(imported, &state, &downloads, &app).await
}

#[tauri::command]
pub async fn settings_reset(state: State<'_, SettingsState>, downloads: State<'_, Downloads>, app: AppHandle) -> CmdResult<AppSettings> {
    replace(AppSettings::default(), &state, &downloads, &app).await
}
