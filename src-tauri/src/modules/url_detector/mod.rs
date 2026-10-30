// URL Detection Module System
use std::fmt;

pub mod builtin;

/// Trait for URL detection modules
pub trait UrlDetector: Send + Sync {
    /// Get the name of this detector
    fn name(&self) -> &str;

    /// Check if this detector can handle the given URL
    fn can_handle(&self, url: &str) -> bool;

    /// Validate and extract information from the URL
    fn validate(&self, url: &str) -> Result<UrlDetectionInfo, DetectionError>;

    /// Get supported file extensions (for file-based detectors)
    fn supported_extensions(&self) -> Vec<&str> {
        vec![]
    }
}

/// Information extracted from URL detection
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlDetectionInfo {
    pub url: String,
    pub is_valid: bool,
    pub detector_name: String,
    pub detection_mode: DetectionMode,
    pub suggested_filename: Option<String>,
    pub file_extension: Option<String>,
}

/// Detection mode type
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DetectionMode {
    FileExtension,
    ModuleCustom,
}

/// Detection error types
#[derive(Debug)]
pub enum DetectionError {
    InvalidUrl(String),
    NoMatchingDetector,
    InternalError(String),
}

impl fmt::Display for DetectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DetectionError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            DetectionError::NoMatchingDetector => write!(f, "No matching detector found"),
            DetectionError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for DetectionError {}

/// Registry for URL detector modules
pub struct ModuleRegistry {
    detectors: Vec<Box<dyn UrlDetector>>,
}

impl ModuleRegistry {
    /// Create a new module registry with builtin detectors
    pub fn new() -> Self {
        let mut registry = Self {
            detectors: vec![],
        };

        // Register builtin file detector
        registry.register(Box::new(builtin::BuiltinFileDetector::new()));

        registry
    }

    /// Register a new URL detector module
    pub fn register(&mut self, detector: Box<dyn UrlDetector>) {
        self.detectors.push(detector);
    }

    /// Detect URL using registered detectors
    pub fn detect_url(&self, url: &str) -> Result<UrlDetectionInfo, DetectionError> {
        // Validate basic URL format
        if !Self::is_valid_url_format(url) {
            return Err(DetectionError::InvalidUrl(url.to_string()));
        }

        // Try each detector until one matches
        for detector in &self.detectors {
            if detector.can_handle(url) {
                return detector.validate(url);
            }
        }

        Err(DetectionError::NoMatchingDetector)
    }

    /// Basic URL format validation
    fn is_valid_url_format(url: &str) -> bool {
        // Check for http/https schema
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return false;
        }

        // Security: reject javascript: and data: schemas
        if url.starts_with("javascript:") || url.starts_with("data:") {
            return false;
        }

        true
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Site Module Integration (Feature 004 - Phase 4.6 - T087)
// ============================================================================

/// Detect which site module (if any) should handle the given URL (T087)
///
/// This function checks the ModuleRegistry to see if any JavaScript module can handle the URL.
/// If no module matches, it falls back to builtin URL detection.
///
/// # Arguments
/// * `url` - The URL to detect
/// * `site_module_registry` - Site module registry containing loaded modules
///
/// # Returns
/// UrlDetectionInfo with detection information
pub fn detect_url_with_modules(
    url: &str,
    site_module_registry: &crate::modules::registry::ModuleRegistry,
) -> UrlDetectionInfo {
    // Step 1: Try to find a site module that can handle this URL
    let module_match = site_module_registry.find_js_module_for_url(url);

    if module_match.is_some() {
        // A site module was found - create result with ModuleCustom mode
        return UrlDetectionInfo {
            url: url.to_string(),
            is_valid: true,
            detector_name: "site-module".to_string(),
            detection_mode: DetectionMode::ModuleCustom,
            suggested_filename: extract_filename_from_url(url),
            file_extension: extract_file_extension(url),
        };
    }

    // Step 2: Fall back to builtin URL detection
    let builtin_registry = ModuleRegistry::new();

    match builtin_registry.detect_url(url) {
        Ok(info) => info,
        Err(_) => {
            // If builtin detection also fails, return invalid result
            UrlDetectionInfo {
                url: url.to_string(),
                is_valid: false,
                detector_name: "none".to_string(),
                detection_mode: DetectionMode::FileExtension,
                suggested_filename: None,
                file_extension: None,
            }
        }
    }
}

/// Extract filename from URL
fn extract_filename_from_url(url: &str) -> Option<String> {
    let parts: Vec<&str> = url.split('/').collect();

    // Get the last non-empty part
    for part in parts.iter().rev() {
        if !part.is_empty() && !part.contains('?') {
            // Remove query parameters if present
            let filename = part.split('?').next().unwrap_or(part);
            if !filename.is_empty() {
                return Some(filename.to_string());
            }
        }
    }

    None
}

/// Extract file extension from URL
fn extract_file_extension(url: &str) -> Option<String> {
    if let Some(filename) = extract_filename_from_url(url) {
        if let Some(dot_pos) = filename.rfind('.') {
            let ext = &filename[dot_pos + 1..];
            if !ext.is_empty() && ext.len() <= 10 {
                // Reasonable extension length
                return Some(ext.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: 確保序列化輸出使用 camelCase
    /// 這個測試確保後端返回的 JSON 格式與前端期望的一致
    #[test]
    fn test_url_detection_info_serialization_uses_camel_case() {
        let info = UrlDetectionInfo {
            url: "https://example.com/test-file.zip".to_string(),
            is_valid: true,
            detector_name: "BuiltinFileDetector".to_string(),
            detection_mode: DetectionMode::FileExtension,
            suggested_filename: Some("test-file.zip".to_string()),
            file_extension: Some("zip".to_string()),
        };

        let json = serde_json::to_string(&info).expect("Failed to serialize");
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("Failed to parse JSON");

        // 驗證所有欄位都使用 camelCase
        assert!(parsed.get("url").is_some(), "Missing 'url' field");
        assert!(
            parsed.get("isValid").is_some(),
            "Missing 'isValid' field (should be camelCase, not is_valid)"
        );
        assert!(
            parsed.get("detectorName").is_some(),
            "Missing 'detectorName' field (should be camelCase, not detector_name)"
        );
        assert!(
            parsed.get("detectionMode").is_some(),
            "Missing 'detectionMode' field (should be camelCase, not detection_mode)"
        );
        assert!(
            parsed.get("suggestedFilename").is_some(),
            "Missing 'suggestedFilename' field (should be camelCase, not suggested_filename)"
        );
        assert!(
            parsed.get("fileExtension").is_some(),
            "Missing 'fileExtension' field (should be camelCase, not file_extension)"
        );

        // 確保沒有 snake_case 欄位
        assert!(
            parsed.get("is_valid").is_none(),
            "Found snake_case field 'is_valid' - should be 'isValid'"
        );
        assert!(
            parsed.get("detector_name").is_none(),
            "Found snake_case field 'detector_name' - should be 'detectorName'"
        );
    }

    /// Test: 確保可以從前端的 camelCase JSON 反序列化
    #[test]
    fn test_url_detection_info_deserialization_from_camel_case() {
        let json = r#"{
            "url": "https://example.com/test-file.zip",
            "isValid": true,
            "detectorName": "BuiltinFileDetector",
            "detectionMode": "fileExtension",
            "suggestedFilename": "test-file.zip",
            "fileExtension": "zip"
        }"#;

        let info: UrlDetectionInfo =
            serde_json::from_str(json).expect("Failed to deserialize from camelCase JSON");

        assert_eq!(info.url, "https://example.com/test-file.zip");
        assert_eq!(info.is_valid, true);
        assert_eq!(info.detector_name, "BuiltinFileDetector");
        assert_eq!(info.suggested_filename, Some("test-file.zip".to_string()));
        assert_eq!(info.file_extension, Some("zip".to_string()));
    }

    /// Test: 確保 enum 也使用 camelCase
    #[test]
    fn test_detection_mode_serialization_uses_camel_case() {
        let mode = DetectionMode::FileExtension;
        let json = serde_json::to_string(&mode).expect("Failed to serialize");

        // FileExtension 應該序列化為 "fileExtension"
        assert_eq!(json, r#""fileExtension""#);

        let mode = DetectionMode::ModuleCustom;
        let json = serde_json::to_string(&mode).expect("Failed to serialize");

        // ModuleCustom 應該序列化為 "moduleCustom"
        assert_eq!(json, r#""moduleCustom""#);
    }

    /// Test: 完整的序列化/反序列化往返測試
    #[test]
    fn test_url_detection_info_roundtrip() {
        let original = UrlDetectionInfo {
            url: "https://example.com/test.pdf".to_string(),
            is_valid: true,
            detector_name: "TestDetector".to_string(),
            detection_mode: DetectionMode::FileExtension,
            suggested_filename: Some("test.pdf".to_string()),
            file_extension: Some("pdf".to_string()),
        };

        // 序列化
        let json = serde_json::to_string(&original).expect("Failed to serialize");

        // 反序列化
        let deserialized: UrlDetectionInfo =
            serde_json::from_str(&json).expect("Failed to deserialize");

        // 驗證資料完整性
        assert_eq!(original.url, deserialized.url);
        assert_eq!(original.is_valid, deserialized.is_valid);
        assert_eq!(original.detector_name, deserialized.detector_name);
        assert_eq!(
            original.suggested_filename,
            deserialized.suggested_filename
        );
        assert_eq!(original.file_extension, deserialized.file_extension);
    }
}
