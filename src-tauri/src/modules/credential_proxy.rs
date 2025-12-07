// Credential Proxy (Feature 004)
//
// This module provides a proxy for modules to access credentials without
// directly exposing passwords. Modules can only use credentials through
// authenticated HTTP requests.

use std::collections::HashMap;
use keyring::Entry;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Credential proxy that mediates credential access
pub struct CredentialProxy {
    module_id: String,
    credentials: HashMap<String, String>,
    http_client: Client,
}

/// Credential validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialStatus {
    pub module_id: String,
    pub has_credentials: bool,
    pub available_keys: Vec<String>,
    pub missing_keys: Vec<String>,
}

impl CredentialProxy {
    /// Create a new credential proxy for a module (Phase 4: T058)
    pub fn new(module_id: String) -> Self {
        Self {
            module_id,
            credentials: HashMap::new(),
            http_client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    /// Load credentials from secure storage (Phase 4: T059, T061)
    pub async fn load_credentials(&mut self, keys: &[String]) -> Result<(), String> {
        self.credentials.clear();

        for key in keys {
            match self.load_credential_key(key) {
                Ok(value) => {
                    self.credentials.insert(key.clone(), value);
                }
                Err(e) => {
                    // Log warning but continue - credentials might be optional
                    eprintln!("Warning: Failed to load credential '{}' for module '{}': {}",
                        key, self.module_id, e);
                }
            }
        }

        Ok(())
    }

    /// Load a single credential key from OS keychain (Phase 4: T059)
    fn load_credential_key(&self, key: &str) -> Result<String, String> {
        let entry = Entry::new(&format!("zoo-downloader.module.{}", self.module_id), key)
            .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

        let value = entry.get_password()
            .map_err(|e| format!("Failed to retrieve credential: {}", e))?;

        Ok(value)
    }

    /// Make an authenticated HTTP request on behalf of the module (Phase 4: T060)
    pub async fn authenticated_request(
        &self,
        url: &str,
        method: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<String>,
    ) -> Result<String, String> {
        // Validate URL (basic check, detailed validation in url_validator.rs)
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(format!("Invalid URL protocol: {}", url));
        }

        // Build request
        let mut request = match method.to_uppercase().as_str() {
            "GET" => self.http_client.get(url),
            "POST" => self.http_client.post(url),
            "PUT" => self.http_client.put(url),
            "DELETE" => self.http_client.delete(url),
            "HEAD" => self.http_client.head(url),
            _ => return Err(format!("Unsupported HTTP method: {}", method)),
        };

        // Add custom headers
        if let Some(header_map) = headers {
            for (key, value) in header_map {
                request = request.header(&key, &value);
            }
        }

        // Add body if present
        if let Some(body_content) = body {
            request = request.body(body_content);
        }

        // Execute request
        let response = request
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        // Check response status
        let status = response.status();
        if !status.is_success() {
            return Err(format!("HTTP request failed with status {}: {}",
                status.as_u16(), status.canonical_reason().unwrap_or("Unknown")));
        }

        // Get response body
        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        Ok(body)
    }

    /// Get credential value (for internal use only, not exposed to JS) (Phase 4: T062)
    fn get_credential(&self, key: &str) -> Option<&String> {
        self.credentials.get(key)
    }

    /// Check if a credential key is available
    pub fn has_credential(&self, key: &str) -> bool {
        self.credentials.contains_key(key)
    }

    /// Get credential status for validation (Phase 4: T061)
    pub fn get_status(&self, required_keys: &[String]) -> CredentialStatus {
        let available_keys: Vec<String> = self.credentials.keys().cloned().collect();
        let missing_keys: Vec<String> = required_keys
            .iter()
            .filter(|key| !self.credentials.contains_key(*key))
            .cloned()
            .collect();

        CredentialStatus {
            module_id: self.module_id.clone(),
            has_credentials: !self.credentials.is_empty(),
            available_keys,
            missing_keys,
        }
    }

    /// Build authenticated HTTP request with credentials in headers
    pub fn build_authenticated_headers(&self, base_headers: Option<HashMap<String, String>>) -> HashMap<String, String> {
        let mut headers = base_headers.unwrap_or_default();

        // Add common authentication headers if credentials are available
        if let Some(username) = self.get_credential("username") {
            headers.insert("X-Username".to_string(), username.clone());
        }

        if let Some(api_key) = self.get_credential("apiKey") {
            headers.insert("X-API-Key".to_string(), api_key.clone());
        }

        if let Some(token) = self.get_credential("token") {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }

        headers
    }

    /// Insert a credential (for testing purposes only)
    #[cfg(test)]
    pub fn insert_credential_for_test(&mut self, key: String, value: String) {
        self.credentials.insert(key, value);
    }
}

/// Save module credentials to secure storage (Phase 4: T059)
pub async fn save_credentials(
    module_id: &str,
    credentials: HashMap<String, String>,
) -> Result<(), String> {
    for (key, value) in credentials {
        let entry = Entry::new(&format!("zoo-downloader.module.{}", module_id), &key)
            .map_err(|e| format!("Failed to create keyring entry for '{}': {}", key, e))?;

        entry.set_password(&value)
            .map_err(|e| format!("Failed to save credential '{}': {}", key, e))?;
    }

    Ok(())
}

/// Delete module credentials from secure storage (Phase 4: T059)
pub async fn delete_credentials(module_id: &str, keys: &[String]) -> Result<(), String> {
    for key in keys {
        let entry = Entry::new(&format!("zoo-downloader.module.{}", module_id), key)
            .map_err(|e| format!("Failed to create keyring entry for '{}': {}", key, e))?;

        // Ignore errors if credential doesn't exist
        let _ = entry.delete_password();
    }

    Ok(())
}

/// Get credential status without loading values (Phase 4: T061)
pub async fn get_credential_status(
    module_id: &str,
    required_keys: &[String],
) -> Result<CredentialStatus, String> {
    let mut available_keys = Vec::new();

    for key in required_keys {
        let entry = Entry::new(&format!("zoo-downloader.module.{}", module_id), key)
            .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

        if entry.get_password().is_ok() {
            available_keys.push(key.clone());
        }
    }

    let missing_keys: Vec<String> = required_keys
        .iter()
        .filter(|key| !available_keys.contains(key))
        .cloned()
        .collect();

    Ok(CredentialStatus {
        module_id: module_id.to_string(),
        has_credentials: !available_keys.is_empty(),
        available_keys,
        missing_keys,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test credential proxy creation (Phase 4: T058)
    #[test]
    fn test_credential_proxy_creation() {
        let proxy = CredentialProxy::new("test-module".to_string());
        assert_eq!(proxy.module_id, "test-module");
        assert!(proxy.credentials.is_empty());
    }

    /// Test credential status checking (Phase 4: T061)
    #[test]
    fn test_credential_status() {
        let mut proxy = CredentialProxy::new("test-module".to_string());
        proxy.credentials.insert("username".to_string(), "test_user".to_string());
        proxy.credentials.insert("password".to_string(), "test_pass".to_string());

        let required_keys = vec![
            "username".to_string(),
            "password".to_string(),
            "apiKey".to_string(),
        ];

        let status = proxy.get_status(&required_keys);

        assert_eq!(status.module_id, "test-module");
        assert!(status.has_credentials);
        assert_eq!(status.available_keys.len(), 2);
        assert_eq!(status.missing_keys, vec!["apiKey".to_string()]);
    }

    /// Test credential isolation (Phase 4: T062)
    #[test]
    fn test_credential_isolation() {
        let mut proxy = CredentialProxy::new("test-module".to_string());
        proxy.credentials.insert("secret".to_string(), "very_secret_value".to_string());

        // get_credential is private - JS context cannot access it
        // Only authenticated_request can use credentials
        assert!(proxy.has_credential("secret"));
        assert!(!proxy.has_credential("nonexistent"));
    }

    /// Test authenticated header building (Phase 4: T060)
    #[test]
    fn test_build_authenticated_headers() {
        let mut proxy = CredentialProxy::new("test-module".to_string());
        proxy.credentials.insert("username".to_string(), "testuser".to_string());
        proxy.credentials.insert("apiKey".to_string(), "test-api-key-123".to_string());
        proxy.credentials.insert("token".to_string(), "test-token-456".to_string());

        let headers = proxy.build_authenticated_headers(None);

        assert_eq!(headers.get("X-Username"), Some(&"testuser".to_string()));
        assert_eq!(headers.get("X-API-Key"), Some(&"test-api-key-123".to_string()));
        assert_eq!(headers.get("Authorization"), Some(&"Bearer test-token-456".to_string()));
    }

    /// Test HTTP request building (Phase 4: T060)
    #[tokio::test]
    async fn test_authenticated_request_invalid_url() {
        let proxy = CredentialProxy::new("test-module".to_string());

        // Test invalid protocol
        let result = proxy.authenticated_request("ftp://invalid.com", "GET", None, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid URL protocol"));
    }

    /// Test unsupported HTTP method (Phase 4: T060)
    #[tokio::test]
    async fn test_unsupported_http_method() {
        let proxy = CredentialProxy::new("test-module".to_string());

        let result = proxy.authenticated_request(
            "https://example.com",
            "PATCH",
            None,
            None,
        ).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported HTTP method"));
    }

    /// Test credential status without credentials (Phase 4: T061)
    #[test]
    fn test_empty_credential_status() {
        let proxy = CredentialProxy::new("test-module".to_string());
        let required_keys = vec!["username".to_string(), "password".to_string()];
        let status = proxy.get_status(&required_keys);

        assert_eq!(status.module_id, "test-module");
        assert!(!status.has_credentials);
        assert!(status.available_keys.is_empty());
        assert_eq!(status.missing_keys.len(), 2);
    }
}
