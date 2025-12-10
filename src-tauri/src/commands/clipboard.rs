// Clipboard command handlers - Tauri commands for clipboard monitoring and URL detection
use crate::modules::url_detector::UrlDetectionInfo;
use crate::UrlDetectorState;
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
    state: State<'_, UrlDetectorState>,
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

/// T144: Detect which module can handle a URL
///
/// Returns module-specific information for URL detection
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlModuleDetectionResult {
    pub url: String,
    pub module_id: Option<String>,
    pub module_name: Option<String>,
    pub is_supported: bool,
    pub requires_auth: bool,
}

#[tauri::command]
pub async fn detect_url_module(
    url: String,
    state: State<'_, UrlDetectorState>,
) -> Result<UrlModuleDetectionResult, String> {
    log::info!("========== detect_url_module called ==========");
    log::info!("URL: {}", url);

    // Validate URL format
    if !is_valid_url(&url) {
        log::error!("✗ Invalid URL format");
        return Err(format!("Invalid URL format: {}", url));
    }

    log::info!("Checking module support...");
    let registry = state.registry.lock().await;
    let detection_result = registry.detect_url(&url);

    let result = match detection_result {
        Ok(info) => {
            // Check if this is a module-based detection (not just file extension)
            let is_module_detection = matches!(
                info.detection_mode,
                crate::modules::url_detector::DetectionMode::ModuleCustom
            );

            log::info!("✓ Detection result:");
            log::info!("  - Valid: {}", info.is_valid);
            log::info!("  - Detector: {}", info.detector_name);
            log::info!("  - Is module: {}", is_module_detection);

            UrlModuleDetectionResult {
                url: info.url.clone(),
                module_id: if is_module_detection {
                    Some(info.detector_name.clone())
                } else {
                    None
                },
                module_name: if is_module_detection {
                    Some(info.detector_name.clone())
                } else {
                    None
                },
                is_supported: info.is_valid,
                requires_auth: is_module_detection, // Assume module detections require auth
            }
        }
        Err(e) => {
            log::warn!("⚠ No module support found: {:?}", e);
            UrlModuleDetectionResult {
                url: url.clone(),
                module_id: None,
                module_name: None,
                is_supported: false,
                requires_auth: false,
            }
        }
    };

    log::info!("========== detect_url_module completed ==========");
    Ok(result)
}

fn is_valid_url(url: &str) -> bool {
    // Basic URL validation (http/https schema, valid format)
    url.starts_with("http://") || url.starts_with("https://")
}
