use tauri::{AppHandle, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::clipboard_watch::OwnCopy;

#[tauri::command]
pub async fn get_clipboard_content(app: AppHandle) -> Result<String, String> {
    app.clipboard().read_text().map_err(|e| format!("Failed to read clipboard: {e}"))
}

/// Put `text` on the clipboard (e.g. a download's link). The clipboard watcher ignores it, so
/// the app does not offer its own copy back as a new download.
#[tauri::command]
pub async fn clipboard_copy(text: String, app: AppHandle) -> Result<(), String> {
    *app.state::<OwnCopy>().0.lock().unwrap() = Some(text.clone());
    app.clipboard().write_text(text).map_err(|e| format!("Failed to write clipboard: {e}"))
}
