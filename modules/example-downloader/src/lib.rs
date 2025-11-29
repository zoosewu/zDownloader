//! Example Website Module for Zoo Downloader
//!
//! This module demonstrates how to implement a WebsiteModule for a file hosting website.
//! It serves as a template for community developers to create their own modules.
//!
//! # Overview
//!
//! This example module shows how to:
//! - Implement the WebsiteModule trait
//! - Parse URLs and extract download information
//! - Handle authentication requirements
//! - Handle CAPTCHA challenges
//! - Extract direct download links from HTML pages
//!
//! # URL Pattern
//!
//! This module handles URLs in the format:
//! - `https://example-filehost.com/file/{file_id}`
//! - `https://example-filehost.com/download/{file_id}`
//!
//! # Usage
//!
//! After building this module:
//! 1. Copy the compiled .so/.dll/.dylib to the modules directory
//! 2. The Zoo Downloader will automatically load it on startup
//! 3. URLs matching this module's pattern will be handled automatically

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export the interface types from the main application
// Note: In a real module, you would need to define these types or link against the main app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub supported_domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub direct_url: String,
    pub filename: String,
    pub filesize: Option<u64>,
    pub mime_type: Option<String>,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaChallenge {
    pub challenge_type: String,
    pub image_url: Option<String>,
    pub site_key: Option<String>,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaSolution {
    pub solution_type: String,
    pub solution_data: String,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCredentials {
    pub username: String,
    pub password: String,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleError {
    #[error("URL not supported: {0}")]
    UnsupportedUrl(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("CAPTCHA required")]
    CaptchaRequired(CaptchaChallenge),
    #[error("Authentication required")]
    AuthRequired,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type ModuleResult<T> = Result<T, ModuleError>;

/// The main trait that all website modules must implement
pub trait WebsiteModule: Send + Sync {
    /// Returns metadata about this module
    fn metadata(&self) -> ModuleMetadata;

    /// Checks if this module can handle the given URL
    fn can_handle(&self, url: &str) -> bool;

    /// Extracts download information from a URL
    ///
    /// This method should:
    /// 1. Parse the webpage at the given URL
    /// 2. Extract the direct download link
    /// 3. Extract file metadata (name, size, type)
    /// 4. Handle authentication if credentials are provided
    /// 5. Return ModuleError::CaptchaRequired if CAPTCHA is needed
    fn extract_download_info(
        &self,
        url: &str,
        credentials: Option<&AuthCredentials>,
    ) -> ModuleResult<DownloadInfo>;

    /// Handles CAPTCHA challenges
    ///
    /// This method is called when extract_download_info returns
    /// ModuleError::CaptchaRequired. After the user provides a solution,
    /// this method validates it.
    fn handle_captcha(
        &self,
        challenge: &CaptchaChallenge,
        solution: &CaptchaSolution,
    ) -> ModuleResult<()>;
}

/// Example implementation for a fictional file hosting website
pub struct ExampleDownloader {
    #[allow(dead_code)] // Client would be used in real implementation for HTTP requests
    client: reqwest::blocking::Client,
    captcha_required: bool,
}

impl ExampleDownloader {
    /// Creates a new instance of the example downloader
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            captcha_required: false,
        }
    }

    /// Extracts file ID from URL
    fn extract_file_id(&self, url: &str) -> Option<String> {
        let re = regex::Regex::new(r"example-filehost\.com/(?:file|download)/([a-zA-Z0-9]+)")
            .unwrap();

        re.captures(url)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Fetches the HTML page and extracts download information
    fn parse_page(&self, url: &str) -> ModuleResult<DownloadInfo> {
        // In a real implementation, you would:
        // 1. Fetch the HTML page
        // 2. Parse it with an HTML parser (like scraper or select)
        // 3. Extract the download link, filename, and metadata

        // For this example, we'll simulate the process
        let file_id = self.extract_file_id(url)
            .ok_or_else(|| ModuleError::ParseError("Invalid URL format".to_string()))?;

        // Simulate checking if CAPTCHA is required
        // In real implementation, this would be based on page content
        if self.captcha_required {
            let challenge = CaptchaChallenge {
                challenge_type: "image".to_string(),
                image_url: Some(format!("https://example-filehost.com/captcha/{}.png", file_id)),
                site_key: None,
                extra: HashMap::new(),
            };
            return Err(ModuleError::CaptchaRequired(challenge));
        }

        // Simulate extracting download info
        // In real implementation, parse HTML to find these values
        let download_info = DownloadInfo {
            direct_url: format!("https://cdn.example-filehost.com/dl/{}", file_id),
            filename: format!("file_{}.zip", file_id),
            filesize: Some(1024 * 1024 * 100), // 100 MB
            mime_type: Some("application/zip".to_string()),
            extra: {
                let mut map = HashMap::new();
                map.insert("file_id".to_string(), file_id);
                map
            },
        };

        Ok(download_info)
    }

    /// Authenticates with provided credentials
    fn authenticate(&self, credentials: &AuthCredentials) -> ModuleResult<()> {
        // In a real implementation, you would:
        // 1. Send login request with credentials
        // 2. Store session cookies in the client
        // 3. Validate the response

        // For this example, we'll just validate that credentials are non-empty
        if credentials.username.is_empty() || credentials.password.is_empty() {
            return Err(ModuleError::InvalidCredentials);
        }

        Ok(())
    }
}

impl Default for ExampleDownloader {
    fn default() -> Self {
        Self::new()
    }
}

impl WebsiteModule for ExampleDownloader {
    fn metadata(&self) -> ModuleMetadata {
        ModuleMetadata {
            id: "example-downloader".to_string(),
            name: "Example File Host Downloader".to_string(),
            version: "0.1.0".to_string(),
            author: "Zoo Downloader Team".to_string(),
            description: "Example module demonstrating WebsiteModule implementation".to_string(),
            supported_domains: vec![
                "example-filehost.com".to_string(),
                "www.example-filehost.com".to_string(),
            ],
        }
    }

    fn can_handle(&self, url: &str) -> bool {
        // Parse URL and check domain
        if let Ok(parsed_url) = url::Url::parse(url) {
            if let Some(domain) = parsed_url.domain() {
                return domain.contains("example-filehost.com");
            }
        }
        false
    }

    fn extract_download_info(
        &self,
        url: &str,
        credentials: Option<&AuthCredentials>,
    ) -> ModuleResult<DownloadInfo> {
        // Authenticate if credentials are provided
        if let Some(creds) = credentials {
            self.authenticate(creds)?;
        }

        // Parse the page and extract download info
        self.parse_page(url)
    }

    fn handle_captcha(
        &self,
        _challenge: &CaptchaChallenge,
        solution: &CaptchaSolution,
    ) -> ModuleResult<()> {
        // In a real implementation, you would:
        // 1. Submit the CAPTCHA solution to the website
        // 2. Validate the response
        // 3. Update session state if successful

        // For this example, we'll just validate that solution is non-empty
        if solution.solution_data.is_empty() {
            return Err(ModuleError::ParseError("Empty CAPTCHA solution".to_string()));
        }

        Ok(())
    }
}

/// Type alias for boxed module instances
pub type BoxedModule = Box<dyn WebsiteModule>;

/// Factory function type for creating module instances
pub type CreateModuleFn = unsafe fn() -> BoxedModule;

/// Creates a new instance of this module
///
/// This is the entry point that the Zoo Downloader will call
/// when loading this module dynamically.
///
/// Note: Box<dyn Trait> is not FFI-safe according to Rust's strict rules,
/// but it works fine for our use case since we control both sides of the FFI boundary.
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn create_module() -> BoxedModule {
    Box::new(ExampleDownloader::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata() {
        let module = ExampleDownloader::new();
        let metadata = module.metadata();

        assert_eq!(metadata.id, "example-downloader");
        assert_eq!(metadata.name, "Example File Host Downloader");
        assert!(metadata.supported_domains.contains(&"example-filehost.com".to_string()));
    }

    #[test]
    fn test_can_handle() {
        let module = ExampleDownloader::new();

        // Should handle these URLs
        assert!(module.can_handle("https://example-filehost.com/file/abc123"));
        assert!(module.can_handle("https://example-filehost.com/download/xyz789"));
        assert!(module.can_handle("https://www.example-filehost.com/file/test"));

        // Should NOT handle these URLs
        assert!(!module.can_handle("https://other-site.com/file/abc123"));
        assert!(!module.can_handle("https://example.com/download/xyz789"));
        assert!(!module.can_handle("invalid-url"));
    }

    #[test]
    fn test_extract_file_id() {
        let module = ExampleDownloader::new();

        assert_eq!(
            module.extract_file_id("https://example-filehost.com/file/abc123"),
            Some("abc123".to_string())
        );
        assert_eq!(
            module.extract_file_id("https://example-filehost.com/download/xyz789"),
            Some("xyz789".to_string())
        );
        assert_eq!(
            module.extract_file_id("https://other-site.com/file/abc123"),
            None
        );
    }

    #[test]
    fn test_extract_download_info_no_auth() {
        let module = ExampleDownloader::new();
        let url = "https://example-filehost.com/file/test123";

        let result = module.extract_download_info(url, None);
        assert!(result.is_ok());

        let info = result.unwrap();
        assert!(info.direct_url.contains("test123"));
        assert!(info.filename.contains("test123"));
        assert_eq!(info.filesize, Some(1024 * 1024 * 100));
    }

    #[test]
    fn test_extract_download_info_with_auth() {
        let module = ExampleDownloader::new();
        let url = "https://example-filehost.com/file/test123";

        let credentials = AuthCredentials {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            extra: HashMap::new(),
        };

        let result = module.extract_download_info(url, Some(&credentials));
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_credentials() {
        let module = ExampleDownloader::new();

        let credentials = AuthCredentials {
            username: "".to_string(),
            password: "".to_string(),
            extra: HashMap::new(),
        };

        let result = module.authenticate(&credentials);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ModuleError::InvalidCredentials));
    }

    #[test]
    fn test_create_module() {
        let module = create_module();
        assert!(module.can_handle("https://example-filehost.com/file/test"));
    }

    #[test]
    fn test_captcha_handling() {
        let module = ExampleDownloader::new();

        let challenge = CaptchaChallenge {
            challenge_type: "image".to_string(),
            image_url: Some("https://example.com/captcha.png".to_string()),
            site_key: None,
            extra: HashMap::new(),
        };

        let solution = CaptchaSolution {
            solution_type: "text".to_string(),
            solution_data: "12345".to_string(),
            extra: HashMap::new(),
        };

        let result = module.handle_captcha(&challenge, &solution);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_captcha_solution() {
        let module = ExampleDownloader::new();

        let challenge = CaptchaChallenge {
            challenge_type: "image".to_string(),
            image_url: None,
            site_key: None,
            extra: HashMap::new(),
        };

        let solution = CaptchaSolution {
            solution_type: "text".to_string(),
            solution_data: "".to_string(),
            extra: HashMap::new(),
        };

        let result = module.handle_captcha(&challenge, &solution);
        assert!(result.is_err());
    }
}
