/// URL validation utilities for security and format checking
use reqwest::Url;

pub fn validate_url_format(url: &str) -> Result<(), String> {
    // Check for empty URL
    if url.trim().is_empty() {
        return Err("URL cannot be empty".to_string());
    }

    // Check schema (http/https only)
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL must use http or https schema".to_string());
    }

    // Check for unsafe schemas (security)
    let lowercase_url = url.to_lowercase();
    if lowercase_url.starts_with("javascript:") || lowercase_url.starts_with("data:") {
        return Err("Unsafe URL schema detected".to_string());
    }

    // Prevent file:// schema
    if lowercase_url.starts_with("file://") {
        return Err("File URLs are not allowed".to_string());
    }

    // Basic URL parsing validation
    match Url::parse(url) {
        Ok(parsed_url) => {
            // Ensure the URL has a valid host
            if parsed_url.host_str().is_none() {
                return Err("URL must have a valid host".to_string());
            }
            Ok(())
        }
        Err(e) => Err(format!("Invalid URL format: {}", e)),
    }
}

/// Extract filename from URL
pub fn extract_filename(url: &str) -> Option<String> {
    match Url::parse(url) {
        Ok(parsed) => {
            if let Some(mut segments) = parsed.path_segments() {
                segments
                    .next_back()
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_format() {
        // Valid URLs
        assert!(validate_url_format("https://example.com/file.zip").is_ok());
        assert!(validate_url_format("http://example.com/file.pdf").is_ok());

        // Invalid schemas
        assert!(validate_url_format("javascript:alert(1)").is_err());
        assert!(validate_url_format("data:text/html,test").is_err());
        assert!(validate_url_format("file:///etc/passwd").is_err());
        assert!(validate_url_format("ftp://example.com/file").is_err());

        // Invalid formats
        assert!(validate_url_format("").is_err());
        assert!(validate_url_format("not a url").is_err());
    }

    #[test]
    fn test_extract_filename() {
        assert_eq!(
            extract_filename("https://example.com/file.zip"),
            Some("file.zip".to_string())
        );
        assert_eq!(
            extract_filename("https://example.com/path/to/document.pdf"),
            Some("document.pdf".to_string())
        );
        assert_eq!(extract_filename("https://example.com/"), None);
    }

}
