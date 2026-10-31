//! Download commands: thin calls into `zoo_core::DownloadManager`.

use crate::commands::error::{CmdResult, CommandError};
use crate::commands::settings::SettingsState;
use crate::downloads::Downloads;
use crate::utils::parser::parse_url_file;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::State;
use ts_rs::TS;
use zoo_core::{AddRequest, SegmentMap, StartMode, Task, TaskId};

fn task_id(id: &str) -> CmdResult<TaskId> {
    Ok(id.parse()?)
}

/// `dir` if given (must be an existing directory), else the default download directory.
async fn target_dir(dir: Option<String>, settings: &SettingsState) -> CmdResult<PathBuf> {
    let dir = match dir.filter(|d| !d.trim().is_empty()) {
        Some(d) => PathBuf::from(d),
        None => settings.settings.lock().await.default_download_dir.clone(),
    };
    ensure_dir(&dir)?;
    Ok(dir)
}

fn ensure_dir(dir: &Path) -> CmdResult<()> {
    std::fs::create_dir_all(dir)
        .map_err(|e| CommandError::new("io", format!("cannot use download folder {}: {e}", dir.display())))
}

#[tauri::command]
pub async fn downloads_add(
    url: String,
    filename: Option<String>,
    dir: Option<String>,
    start: StartMode,
    downloads: State<'_, Downloads>,
    settings: State<'_, SettingsState>,
) -> CmdResult<Task> {
    let dir = target_dir(dir, &settings).await?;
    Ok(downloads.manager.add(AddRequest { url, filename, dir, start }).await?)
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct BatchResult {
    pub added: Vec<Task>,
    /// `(url, reason)` for every URL that could not be added.
    pub failed: Vec<(String, String)>,
}

#[tauri::command]
pub async fn downloads_import_batch(
    file_path: String,
    dir: Option<String>,
    start: StartMode,
    downloads: State<'_, Downloads>,
    settings: State<'_, SettingsState>,
) -> CmdResult<BatchResult> {
    let urls = parse_url_file(Path::new(&file_path)).map_err(CommandError::invalid)?;
    let dir = target_dir(dir, &settings).await?;
    let mut result = BatchResult { added: Vec::new(), failed: Vec::new() };
    for url in urls {
        let req = AddRequest { url: url.clone(), filename: None, dir: dir.clone(), start };
        match downloads.manager.add(req).await {
            Ok(task) => result.added.push(task),
            Err(e) => result.failed.push((url, e.to_string())),
        }
    }
    Ok(result)
}

#[tauri::command]
pub async fn downloads_list(downloads: State<'_, Downloads>) -> CmdResult<Vec<Task>> {
    Ok(downloads.manager.list().await?)
}

#[tauri::command]
pub async fn downloads_pause(id: String, downloads: State<'_, Downloads>) -> CmdResult<Task> {
    Ok(downloads.manager.pause(task_id(&id)?).await?)
}

#[tauri::command]
pub async fn downloads_resume(id: String, downloads: State<'_, Downloads>) -> CmdResult<Task> {
    Ok(downloads.manager.resume(task_id(&id)?).await?)
}

#[tauri::command]
pub async fn downloads_cancel(id: String, downloads: State<'_, Downloads>) -> CmdResult<Task> {
    Ok(downloads.manager.cancel(task_id(&id)?).await?)
}

#[tauri::command]
pub async fn downloads_retry(id: String, downloads: State<'_, Downloads>) -> CmdResult<Task> {
    Ok(downloads.manager.retry(task_id(&id)?).await?)
}

#[tauri::command]
pub async fn downloads_remove(id: String, downloads: State<'_, Downloads>) -> CmdResult<()> {
    Ok(downloads.manager.remove(task_id(&id)?).await?)
}

#[tauri::command]
pub async fn downloads_pause_all(downloads: State<'_, Downloads>) -> CmdResult<Vec<Task>> {
    Ok(downloads.manager.pause_all().await?)
}

#[tauri::command]
pub async fn downloads_resume_all(downloads: State<'_, Downloads>) -> CmdResult<Vec<Task>> {
    Ok(downloads.manager.resume_all().await?)
}

#[tauri::command]
pub async fn downloads_clear_finished(downloads: State<'_, Downloads>) -> CmdResult<Vec<TaskId>> {
    Ok(downloads.manager.clear_finished().await?)
}

/// Done and in-flight byte ranges, for the segment view (polled while it is open).
#[tauri::command]
pub async fn downloads_segments(id: String, downloads: State<'_, Downloads>) -> CmdResult<SegmentMap> {
    Ok(downloads.manager.segments(task_id(&id)?).await?)
}
