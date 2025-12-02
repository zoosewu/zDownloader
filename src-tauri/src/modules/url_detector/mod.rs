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
pub struct UrlDetectionInfo {
    pub url: String,
    pub is_valid: bool,
    pub detector_name: String,
    pub detection_mode: DetectionMode,
    pub suggested_filename: Option<String>,
    pub file_extension: Option<String>,
}

/// Detection mode type
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
