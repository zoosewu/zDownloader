//! Download commands: thin calls into `zoo_core::DownloadManager`.

use crate::commands::error::{CmdResult, CommandError};
use crate::commands::settings::SettingsState;
use crate::downloads::Downloads;
use std::path::{Path, PathBuf};
use tauri::State;
use zoo_core::{AddRequest, SegmentMap, StartMode, Task, TaskId, TaskUpdate};

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
    headers: Option<Vec<(String, String)>>,
    tags: Option<Vec<String>>,
    downloads: State<'_, Downloads>,
    settings: State<'_, SettingsState>,
) -> CmdResult<Task> {
    let dir = target_dir(dir, &settings).await?;
    let req = AddRequest { url, filename, dir, start, headers: headers.unwrap_or_default(), tags };
    Ok(downloads.manager.add(req).await?)
}

/// A download from the browser extension, started at once with the default folder.
pub async fn add_now(
    downloads: &Downloads,
    settings: &SettingsState,
    url: String,
    filename: Option<String>,
    headers: Vec<(String, String)>,
) -> CmdResult<Task> {
    let dir = target_dir(None, settings).await?;
    Ok(downloads.manager.add(AddRequest { url, filename, dir, start: StartMode::Now, headers, tags: None }).await?)
}

/// The categories a download would get from the rules (to pre-fill the add window).
#[tauri::command]
pub async fn categories_match(url: String, filename: Option<String>, settings: State<'_, SettingsState>) -> CmdResult<Vec<String>> {
    let categories = settings.settings.lock().await.categories.clone();
    let name = zoo_core::naming::filename_for(filename.as_deref(), &url);
    Ok(zoo_core::Classifier::new(&categories)?.tags(&url, &name))
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

/// Discard the partial data and download from the start.
#[tauri::command]
pub async fn downloads_restart(id: String, downloads: State<'_, Downloads>) -> CmdResult<Task> {
    Ok(downloads.manager.restart(task_id(&id)?).await?)
}

/// Change the URL or headers of a task that is not running.
#[tauri::command]
pub async fn downloads_update(id: String, update: TaskUpdate, downloads: State<'_, Downloads>) -> CmdResult<Task> {
    Ok(downloads.manager.update(task_id(&id)?, update).await?)
}

/// Put a task in exactly these categories; their rules no longer change it.
#[tauri::command]
pub async fn downloads_set_tags(id: String, tags: Vec<String>, downloads: State<'_, Downloads>) -> CmdResult<Task> {
    Ok(downloads.manager.set_tags(task_id(&id)?, tags).await?)
}

#[tauri::command]
pub async fn downloads_remove(id: String, downloads: State<'_, Downloads>) -> CmdResult<()> {
    Ok(downloads.manager.remove(task_id(&id)?).await?)
}

/// Move a task in front of `before` (or to the end); queued tasks start in list order.
#[tauri::command]
pub async fn downloads_move(id: String, before: Option<String>, downloads: State<'_, Downloads>) -> CmdResult<Task> {
    let before = before.as_deref().map(task_id).transpose()?;
    Ok(downloads.manager.move_before(task_id(&id)?, before).await?)
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
