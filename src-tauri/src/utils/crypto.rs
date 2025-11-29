// Credential encryption using OS Keychain
use keyring::Entry;

/// Error type for credential operations
#[derive(Debug)]
pub enum CredentialError {
    /// Failed to access keychain
    KeychainError(String),
    /// Credential not found
    NotFound(String),
    /// Invalid credential format
    InvalidFormat(String),
}

impl std::fmt::Display for CredentialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CredentialError::KeychainError(msg) => write!(f, "Keychain error: {}", msg),
            CredentialError::NotFound(msg) => write!(f, "Credential not found: {}", msg),
            CredentialError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}

impl std::error::Error for CredentialError {}

impl From<keyring::Error> for CredentialError {
    fn from(err: keyring::Error) -> Self {
        match err {
            keyring::Error::NoEntry => {
                CredentialError::NotFound("Credential not found in keychain".to_string())
            }
            _ => CredentialError::KeychainError(err.to_string()),
        }
    }
}

pub type CredentialResult<T> = Result<T, CredentialError>;

/// Credential storage using OS Keychain
///
/// This module uses the operating system's secure credential storage:
/// - Windows: Windows Credential Manager
/// - macOS: Keychain
/// - Linux: Secret Service API (libsecret)
pub struct CredentialStore {
    service_name: String,
}

impl CredentialStore {
    /// Creates a new credential store
    ///
    /// # Arguments
    /// * `service_name` - The service name for keychain entries (e.g., "zoo-downloader")
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }

    /// Stores a credential in the OS keychain
    ///
    /// # Arguments
    /// * `username` - The username/identifier
    /// * `password` - The password/secret to store
    ///
    /// # Returns
    /// Ok(()) if successful
    pub fn store(&self, username: &str, password: &str) -> CredentialResult<()> {
        let entry = Entry::new(&self.service_name, username)?;
        entry.set_password(password)?;
        Ok(())
    }

    /// Retrieves a credential from the OS keychain
    ///
    /// # Arguments
    /// * `username` - The username/identifier
    ///
    /// # Returns
    /// The password/secret if found
    pub fn retrieve(&self, username: &str) -> CredentialResult<String> {
        let entry = Entry::new(&self.service_name, username)?;
        let password = entry.get_password()?;
        Ok(password)
    }

    /// Deletes a credential from the OS keychain
    ///
    /// # Arguments
    /// * `username` - The username/identifier
    ///
    /// # Returns
    /// Ok(()) if successful
    pub fn delete(&self, username: &str) -> CredentialResult<()> {
        let entry = Entry::new(&self.service_name, username)?;
        entry.delete_password()?;
        Ok(())
    }

    /// Checks if a credential exists in the OS keychain
    ///
    /// # Arguments
    /// * `username` - The username/identifier
    ///
    /// # Returns
    /// true if the credential exists, false otherwise
    pub fn exists(&self, username: &str) -> bool {
        self.retrieve(username).is_ok()
    }

    /// Stores multiple key-value pairs as a JSON string
    ///
    /// # Arguments
    /// * `username` - The username/identifier
    /// * `data` - Map of key-value pairs to store
    ///
    /// # Returns
    /// Ok(()) if successful
    pub fn store_json(
        &self,
        username: &str,
        data: &std::collections::HashMap<String, String>,
    ) -> CredentialResult<()> {
        let json = serde_json::to_string(data)
            .map_err(|e| CredentialError::InvalidFormat(e.to_string()))?;

        self.store(username, &json)
    }

    /// Retrieves multiple key-value pairs from JSON string
    ///
    /// # Arguments
    /// * `username` - The username/identifier
    ///
    /// # Returns
    /// Map of key-value pairs if found and valid JSON
    pub fn retrieve_json(
        &self,
        username: &str,
    ) -> CredentialResult<std::collections::HashMap<String, String>> {
        let json = self.retrieve(username)?;

        let data = serde_json::from_str(&json)
            .map_err(|e| CredentialError::InvalidFormat(e.to_string()))?;

        Ok(data)
    }
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new("zoo-downloader")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_store_creation() {
        let store = CredentialStore::new("test-app");
        assert_eq!(store.service_name, "test-app");
    }

    #[test]
    fn test_credential_store_default() {
        let store = CredentialStore::default();
        assert_eq!(store.service_name, "zoo-downloader");
    }

    #[test]
    fn test_credential_error_display() {
        let error = CredentialError::KeychainError("test error".to_string());
        assert!(error.to_string().contains("Keychain error"));

        let error = CredentialError::NotFound("test".to_string());
        assert!(error.to_string().contains("not found"));
    }

    // Note: Actual keychain tests require OS keychain services
    // These tests are skipped in CI environments
    #[test]
    #[ignore]
    fn test_store_and_retrieve() {
        let store = CredentialStore::new("zoo-downloader-test");
        let test_user = "test_user_12345";
        let test_pass = "test_password";

        // Store
        store.store(test_user, test_pass).unwrap();

        // Retrieve
        let retrieved = store.retrieve(test_user).unwrap();
        assert_eq!(retrieved, test_pass);

        // Clean up
        store.delete(test_user).unwrap();
    }

    #[test]
    #[ignore]
    fn test_exists() {
        let store = CredentialStore::new("zoo-downloader-test");
        let test_user = "test_user_exists";
        let test_pass = "test_password";

        // Should not exist initially
        assert!(!store.exists(test_user));

        // Store and check
        store.store(test_user, test_pass).unwrap();
        assert!(store.exists(test_user));

        // Clean up
        store.delete(test_user).unwrap();
        assert!(!store.exists(test_user));
    }

    #[test]
    #[ignore]
    fn test_json_storage() {
        let store = CredentialStore::new("zoo-downloader-test");
        let test_user = "test_user_json";

        let mut data = std::collections::HashMap::new();
        data.insert("api_key".to_string(), "secret123".to_string());
        data.insert("token".to_string(), "xyz789".to_string());

        // Store JSON
        store.store_json(test_user, &data).unwrap();

        // Retrieve JSON
        let retrieved = store.retrieve_json(test_user).unwrap();
        assert_eq!(retrieved.get("api_key").unwrap(), "secret123");
        assert_eq!(retrieved.get("token").unwrap(), "xyz789");

        // Clean up
        store.delete(test_user).unwrap();
    }
}
