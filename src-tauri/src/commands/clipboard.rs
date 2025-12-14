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

    log::info!("Checking JavaScript modules...");

    // Step 1: Load JavaScript modules from disk and check for match
    // Do this in a scope to ensure ModuleRegistry is dropped before await
    let js_module_result = {
        let modules_dir = crate::commands::module_management::find_modules_directory();
        log::info!("📁 Modules directory: {:?}", modules_dir);

        let mut js_module_registry = crate::modules::registry::ModuleRegistry::with_path(&modules_dir);
        let loaded_count = js_module_registry.scan_and_register(&modules_dir);
        log::info!("📦 Loaded {} JavaScript module(s)", loaded_count);

        // Check if any JavaScript module can handle this URL
        let module_id = js_module_registry.find_js_module_for_url(&url);

        if let Some(id) = module_id {
            log::info!("✓ JavaScript module found: {}", id);

            // Get full manifest for the module
            let manifests = js_module_registry.get_all_js_module_manifests();
            if let Some(manifest) = manifests.iter().find(|m| m.id == id) {
                log::info!("  - Module name: {}", manifest.name);
                log::info!("  - Requires credentials: {}", manifest.permissions.credentials.is_some());

                Some(UrlModuleDetectionResult {
                    url: url.clone(),
                    module_id: Some(manifest.id.clone()),
                    module_name: Some(manifest.name.clone()),
                    is_supported: true,
                    requires_auth: manifest.permissions.credentials.is_some(),
                })
            } else {
                log::warn!("⚠ Module ID '{}' found but manifest not available", id);
                None
            }
        } else {
            None
        }
    }; // ModuleRegistry is dropped here

    // Step 2: If we found a JS module, return it
    if let Some(result) = js_module_result {
        log::info!("========== detect_url_module completed (JS module) ==========");
        return Ok(result);
    }

    // Step 3: Fall back to builtin URL detection
    log::info!("No JavaScript module found, trying builtin detectors...");
    let registry = state.registry.lock().await;
    let detection_result = registry.detect_url(&url);

    let result = match detection_result {
        Ok(info) => {
            // Check if this is a module-based detection (not just file extension)
            let is_module_detection = matches!(
                info.detection_mode,
                crate::modules::url_detector::DetectionMode::ModuleCustom
            );

            log::info!("✓ Builtin detection result:");
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
            log::warn!("⚠ No builtin module support found: {:?}", e);
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
