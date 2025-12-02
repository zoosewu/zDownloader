// Builtin File Extension Detector
use super::{DetectionError, DetectionMode, UrlDetectionInfo, UrlDetector};
use crate::utils::url_validator;

/// Builtin detector for common file extensions
pub struct BuiltinFileDetector {
    extensions: Vec<String>,
}

impl BuiltinFileDetector {
    /// Create a new builtin file detector with common extensions
    pub fn new() -> Self {
        Self {
            extensions: vec![
                ".zip", ".pdf", ".exe", ".mp4", ".mkv", ".rar", ".7z", ".tar", ".gz", ".iso",
                ".dmg", ".apk",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        }
    }

    /// Extract filename from URL using url_validator utility
    fn extract_filename(&self, url: &str) -> Option<String> {
        url_validator::extract_filename(url)
    }

    /// Get file extension from filename using url_validator utility
    fn get_file_extension(&self, filename: &str) -> Option<String> {
        filename.rfind('.').map(|idx| filename[idx..].to_string())
    }
}

impl Default for BuiltinFileDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl UrlDetector for BuiltinFileDetector {
    fn name(&self) -> &str {
        "BuiltinFileDetector"
    }

    fn can_handle(&self, url: &str) -> bool {
        // Check if URL ends with any supported extension
        self.extensions
            .iter()
            .any(|ext| url.to_lowercase().contains(&ext.to_lowercase()))
    }

    fn validate(&self, url: &str) -> Result<UrlDetectionInfo, DetectionError> {
        // Validate URL format first
        url_validator::validate_url_format(url)
            .map_err(DetectionError::InvalidUrl)?;

        // Extract filename
        let filename = self.extract_filename(url).ok_or_else(|| {
            DetectionError::InternalError("Failed to extract filename from URL".to_string())
        })?;

        // Get extension
        let extension = self.get_file_extension(&filename).ok_or_else(|| {
            DetectionError::InternalError("No file extension found".to_string())
        })?;

        // Verify extension is supported
        let ext_lower = extension.to_lowercase();
        if !self.extensions.iter().any(|e| e.to_lowercase() == ext_lower) {
            return Ok(UrlDetectionInfo {
                url: url.to_string(),
                is_valid: false,
                detector_name: self.name().to_string(),
                detection_mode: DetectionMode::FileExtension,
                suggested_filename: None,
                file_extension: None,
            });
        }

        // Success - valid download URL
        Ok(UrlDetectionInfo {
            url: url.to_string(),
            is_valid: true,
            detector_name: self.name().to_string(),
            detection_mode: DetectionMode::FileExtension,
            suggested_filename: Some(filename.clone()),
            file_extension: Some(extension),
        })
    }

    fn supported_extensions(&self) -> Vec<&str> {
        self.extensions.iter().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle_zip() {
        let detector = BuiltinFileDetector::new();
        assert!(detector.can_handle("https://example.com/file.zip"));
    }

    #[test]
    fn test_can_handle_pdf() {
        let detector = BuiltinFileDetector::new();
        assert!(detector.can_handle("https://example.com/document.pdf"));
    }

    #[test]
    fn test_cannot_handle_html() {
        let detector = BuiltinFileDetector::new();
        assert!(!detector.can_handle("https://example.com/page.html"));
    }

    #[test]
    fn test_extract_filename() {
        let detector = BuiltinFileDetector::new();
        let filename = detector.extract_filename("https://example.com/path/file.zip");
        assert_eq!(filename, Some("file.zip".to_string()));
    }

    #[test]
    fn test_extract_filename_with_query() {
        let detector = BuiltinFileDetector::new();
        let filename = detector.extract_filename("https://example.com/file.zip?download=true");
        assert_eq!(filename, Some("file.zip".to_string()));
    }

    #[test]
    fn test_get_extension() {
        let detector = BuiltinFileDetector::new();
        let ext = detector.get_file_extension("file.zip");
        assert_eq!(ext, Some(".zip".to_string()));
    }

    #[test]
    fn test_validate_success() {
        let detector = BuiltinFileDetector::new();
        let result = detector.validate("https://example.com/file.zip").unwrap();
        assert!(result.is_valid);
        assert_eq!(result.suggested_filename, Some("file.zip".to_string()));
        assert_eq!(result.file_extension, Some(".zip".to_string()));
    }
}
