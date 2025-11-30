// WebsiteModule trait definition for dynamic website support
#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata about a website module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMetadata {
    /// Unique identifier for the module (e.g., "mediafire", "mega")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Module version
    pub version: String,
    /// Module author
    pub author: String,
    /// Supported URL patterns (regex patterns)
    pub url_patterns: Vec<String>,
    /// Whether this module requires authentication
    pub requires_auth: bool,
    /// Whether this module supports CAPTCHA handling
    pub supports_captcha: bool,
}

/// Download information extracted from a webpage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    /// Direct download URL
    pub download_url: String,
    /// Filename (if available)
    pub filename: Option<String>,
    /// File size in bytes (if available)
    pub file_size: Option<u64>,
    /// Additional headers needed for download
    pub headers: HashMap<String, String>,
    /// Whether authentication is required
    pub requires_auth: bool,
}

/// CAPTCHA challenge information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaChallenge {
    /// CAPTCHA type (e.g., "image", "recaptcha", "hcaptcha")
    pub captcha_type: String,
    /// CAPTCHA image URL or site key
    pub challenge_data: String,
    /// Additional parameters
    pub params: HashMap<String, String>,
}

/// CAPTCHA solution from user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaSolution {
    /// Solution text or token
    pub solution: String,
    /// Additional data
    pub metadata: HashMap<String, String>,
}

/// Authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCredentials {
    /// Username or email
    pub username: String,
    /// Password
    pub password: String,
    /// Additional auth data (API keys, tokens, etc.)
    pub extra: HashMap<String, String>,
}

/// Result type for module operations
pub type ModuleResult<T> = Result<T, ModuleError>;

/// Errors that can occur in module operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModuleError {
    /// URL not supported by this module
    UnsupportedUrl(String),
    /// Network error
    NetworkError(String),
    /// CAPTCHA required
    CaptchaRequired(CaptchaChallenge),
    /// Authentication required
    AuthRequired,
    /// Invalid credentials
    InvalidCredentials,
    /// Parsing error
    ParseError(String),
    /// Module internal error
    InternalError(String),
}

impl std::fmt::Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleError::UnsupportedUrl(url) => write!(f, "Unsupported URL: {}", url),
            ModuleError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ModuleError::CaptchaRequired(_) => write!(f, "CAPTCHA verification required"),
            ModuleError::AuthRequired => write!(f, "Authentication required"),
            ModuleError::InvalidCredentials => write!(f, "Invalid credentials"),
            ModuleError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ModuleError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ModuleError {}

/// The main trait that all website support modules must implement
///
/// This trait defines the interface for dynamically loadable website modules
/// that can extract download information from various file hosting sites.
pub trait WebsiteModule: Send + Sync {
    /// Returns metadata about this module
    fn metadata(&self) -> ModuleMetadata;

    /// Checks if this module can handle the given URL
    ///
    /// # Arguments
    /// * `url` - The URL to check
    ///
    /// # Returns
    /// `true` if this module can handle the URL, `false` otherwise
    fn can_handle(&self, url: &str) -> bool;

    /// Extracts download information from a webpage URL
    ///
    /// This is the main function that parses the webpage and extracts
    /// the actual download URL and metadata.
    ///
    /// # Arguments
    /// * `url` - The webpage URL to extract from
    /// * `credentials` - Optional authentication credentials
    ///
    /// # Returns
    /// `DownloadInfo` containing the direct download URL and metadata,
    /// or a `ModuleError` if extraction fails
    fn extract_download_info(
        &self,
        url: &str,
        credentials: Option<&AuthCredentials>,
    ) -> ModuleResult<DownloadInfo>;

    /// Handles CAPTCHA challenges
    ///
    /// This function attempts to solve a CAPTCHA challenge.
    /// It may use various strategies (OCR, API services, user input).
    ///
    /// # Arguments
    /// * `challenge` - The CAPTCHA challenge to solve
    /// * `solution` - The solution provided (usually from user input)
    ///
    /// # Returns
    /// `Ok(())` if CAPTCHA was solved successfully, or an error
    fn handle_captcha(
        &self,
        challenge: &CaptchaChallenge,
        solution: &CaptchaSolution,
    ) -> ModuleResult<()>;

    /// Optional: Perform any cleanup when the module is unloaded
    fn cleanup(&self) -> ModuleResult<()> {
        Ok(())
    }
}

/// Type alias for boxed modules
pub type BoxedModule = Box<dyn WebsiteModule>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_error_display() {
        let error = ModuleError::UnsupportedUrl("http://example.com".to_string());
        assert!(error.to_string().contains("Unsupported URL"));

        let error = ModuleError::NetworkError("Connection timeout".to_string());
        assert!(error.to_string().contains("Network error"));

        let error = ModuleError::AuthRequired;
        assert_eq!(error.to_string(), "Authentication required");
    }

    #[test]
    fn test_metadata_serialization() {
        let metadata = ModuleMetadata {
            id: "test-module".to_string(),
            name: "Test Module".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            url_patterns: vec!["https://example.com/.*".to_string()],
            requires_auth: false,
            supports_captcha: true,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: ModuleMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(metadata.id, deserialized.id);
        assert_eq!(metadata.name, deserialized.name);
        assert_eq!(metadata.version, deserialized.version);
    }
}
