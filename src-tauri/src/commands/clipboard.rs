use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub async fn get_clipboard_content(app: AppHandle) -> Result<String, String> {
    app.clipboard().read_text().map_err(|e| format!("Failed to read clipboard: {e}"))
}
