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
    log::info!("========== detect_url called ==========");
    log::info!("URL: {}", url);

    // Validate URL format
    log::info!("Validating URL format...");
    if !is_valid_url(&url) {
        log::error!("✗ Invalid URL format");
        return Err(format!("Invalid URL format: {}", url));
    }
    log::info!("✓ URL format is valid");

    log::info!("Checking URL detection modules...");
    let registry = state.registry.lock().await;
    let result = registry
        .detect_url(&url)
        .map_err(|e| {
            log::error!("✗ Detection failed: {:?}", e);
            format!("Detection failed: {:?}", e)
        });

    if let Ok(ref info) = result {
        log::info!("✓ URL detection completed:");
        log::info!("  - Valid: {}", info.is_valid);
        log::info!("  - Detector: {}", info.detector_name);
        log::info!("  - Mode: {:?}", info.detection_mode);
        log::info!("  - Suggested filename: {:?}", info.suggested_filename);
    }

    log::info!("========== detect_url completed ==========");
    result
}

fn is_valid_url(url: &str) -> bool {
    // Basic URL validation (http/https schema, valid format)
    url.starts_with("http://") || url.starts_with("https://")
}
