//! Download commands: thin calls into `zoo_core::DownloadManager`.

use crate::commands::error::{CmdResult, CommandError};
use crate::commands::settings::SettingsState;
use crate::downloads::Downloads;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;
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

/// `dir` plus a folder inside a share (`a/b`), each part made a safe name, so nothing a site
/// says can lead outside `dir`.
fn inside(dir: PathBuf, folder: Option<&str>) -> PathBuf {
    folder
        .into_iter()
        .flat_map(|f| f.split(['/', '\\']))
        .filter(|part| !matches!(part.trim(), "" | "." | ".."))
        .fold(dir, |dir, part| dir.join(zoo_core::naming::sanitize(part)))
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
    folder: Option<String>,
    answers: Option<BTreeMap<String, String>>,
    downloads: State<'_, Downloads>,
    settings: State<'_, SettingsState>,
) -> CmdResult<Task> {
    let dir = inside(target_dir(dir, &settings).await?, folder.as_deref());
    ensure_dir(&dir)?;
    let req = AddRequest { url, filename, dir, start, headers: headers.unwrap_or_default(), tags, answers: answers.unwrap_or_default() };
    Ok(downloads.manager.add(req).await?)
}

/// A download from the browser extension, started at once with the default folder.
pub async fn add_now(
    downloads: &Downloads,
    settings: &SettingsState,
    url: String,
    filename: Option<String>,
    headers: Vec<(String, String)>,
    answers: BTreeMap<String, String>,
) -> CmdResult<Task> {
    let dir = target_dir(None, settings).await?;
    Ok(downloads.manager.add(AddRequest { url, filename, dir, start: StartMode::Now, headers, tags: None, answers }).await?)
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
pub async fn downloads_remove(id: String, delete_file: Option<bool>, downloads: State<'_, Downloads>) -> CmdResult<()> {
    // Removing discards a partial download; `delete_file` also takes a finished one, to the
    // recycle bin so a slip can be undone.
    let file = match delete_file {
        Some(true) => finished_file(&downloads, &id).await.ok(),
        _ => None,
    };
    downloads.manager.remove(task_id(&id)?).await?;
    if let Some(path) = file {
        trash::delete(&path)
            .map_err(|e| CommandError::new("io", format!("removed, but cannot move {} to the recycle bin: {e}", path.display())))?;
    }
    Ok(())
}

/// Open a finished download with its default app.
#[tauri::command]
pub async fn downloads_open_file(id: String, app: AppHandle, downloads: State<'_, Downloads>) -> CmdResult<()> {
    let path = finished_file(&downloads, &id).await?;
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|e| CommandError::new("io", format!("cannot open {}: {e}", path.display())))
}

/// Show a finished download in its folder, selected.
#[tauri::command]
pub async fn downloads_show_in_folder(id: String, app: AppHandle, downloads: State<'_, Downloads>) -> CmdResult<()> {
    let path = finished_file(&downloads, &id).await?;
    app.opener()
        .reveal_item_in_dir(&path)
        .map_err(|e| CommandError::new("io", format!("cannot show {}: {e}", path.display())))
}

/// The file a finished task wrote, if it is still there. Only these paths are opened: the page
/// never names a path itself.
async fn finished_file(downloads: &Downloads, id: &str) -> CmdResult<PathBuf> {
    let id = task_id(id)?;
    let task = downloads.manager.list().await?.into_iter().find(|t| t.id == id);
    task.and_then(|t| t.final_path)
        .filter(|path| path.is_file())
        .ok_or_else(|| CommandError::new("notFound", "the downloaded file is not there any more"))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn share_folders_stay_inside_the_download_folder() {
        let base = PathBuf::from("dl");
        assert_eq!(inside(base.clone(), None), base);
        assert_eq!(inside(base.clone(), Some("罚抄存档/校园怪兽")), base.join("罚抄存档").join("校园怪兽"));
        assert_eq!(inside(base.clone(), Some("/a//b/")), base.join("a").join("b"));
        // Nothing climbs out or names a drive or a device.
        assert_eq!(inside(base.clone(), Some("../../x")), base.join("x"));
        assert_eq!(inside(base.clone(), Some("C:/AUX/a<b")), base.join("C").join("_AUX").join("ab"));
    }
}
