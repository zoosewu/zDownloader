// Clipboard command handlers - Tauri commands for clipboard monitoring and URL detection
use crate::modules::url_detector::UrlDetectionInfo;
use crate::ModuleRegistryState;
use tauri::{AppHandle, State};
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub async fn get_clipboard_content(app: AppHandle) -> Result<String, String> {
    app.clipboard()
        .read_text()
        .map_err(|e| format!("Failed to read clipboard: {}", e))
}

#[tauri::command]
pub async fn detect_url(
    url: String,
    state: State<'_, ModuleRegistryState>,
) -> Result<UrlDetectionInfo, String> {
    // Validate URL format
    if !is_valid_url(&url) {
        return Err(format!("Invalid URL format: {}", url));
    }

    let registry = state.registry.lock().await;
    registry
        .detect_url(&url)
        .map_err(|e| format!("Detection failed: {:?}", e))
}

fn is_valid_url(url: &str) -> bool {
    // Basic URL validation (http/https schema, valid format)
    url.starts_with("http://") || url.starts_with("https://")
}
