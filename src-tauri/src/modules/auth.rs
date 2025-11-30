// Authentication credentials management for website modules
#![allow(dead_code)]
use crate::utils::crypto::{CredentialError, CredentialResult, CredentialStore};
use super::interface::AuthCredentials;
use std::collections::HashMap;

/// Manager for website module authentication credentials
pub struct AuthManager {
    store: CredentialStore,
}

impl AuthManager {
    /// Creates a new authentication manager
    pub fn new() -> Self {
        Self {
            store: CredentialStore::default(),
        }
    }

    /// Stores credentials for a website module
    ///
    /// # Arguments
    /// * `module_id` - The module identifier (e.g., "mediafire", "mega")
    /// * `credentials` - The authentication credentials to store
    ///
    /// # Returns
    /// Ok(()) if successful
    pub fn store_credentials(
        &self,
        module_id: &str,
        credentials: &AuthCredentials,
    ) -> CredentialResult<()> {
        let mut data = HashMap::new();
        data.insert("username".to_string(), credentials.username.clone());
        data.insert("password".to_string(), credentials.password.clone());

        // Store extra fields
        for (key, value) in &credentials.extra {
            data.insert(format!("extra_{}", key), value.clone());
        }

        self.store.store_json(module_id, &data)
    }

    /// Retrieves credentials for a website module
    ///
    /// # Arguments
    /// * `module_id` - The module identifier
    ///
    /// # Returns
    /// The authentication credentials if found
    pub fn retrieve_credentials(&self, module_id: &str) -> CredentialResult<AuthCredentials> {
        let data = self.store.retrieve_json(module_id)?;

        let username = data
            .get("username")
            .cloned()
            .ok_or_else(|| CredentialError::InvalidFormat("Missing username".to_string()))?;

        let password = data
            .get("password")
            .cloned()
            .ok_or_else(|| CredentialError::InvalidFormat("Missing password".to_string()))?;

        // Extract extra fields
        let mut extra = HashMap::new();
        for (key, value) in data {
            if let Some(extra_key) = key.strip_prefix("extra_") {
                extra.insert(extra_key.to_string(), value);
            }
        }

        Ok(AuthCredentials {
            username,
            password,
            extra,
        })
    }

    /// Deletes credentials for a website module
    ///
    /// # Arguments
    /// * `module_id` - The module identifier
    ///
    /// # Returns
    /// Ok(()) if successful
    pub fn delete_credentials(&self, module_id: &str) -> CredentialResult<()> {
        self.store.delete(module_id)
    }

    /// Checks if credentials exist for a website module
    ///
    /// # Arguments
    /// * `module_id` - The module identifier
    ///
    /// # Returns
    /// true if credentials exist, false otherwise
    pub fn has_credentials(&self, module_id: &str) -> bool {
        self.store.exists(module_id)
    }

    /// Updates specific fields in stored credentials
    ///
    /// # Arguments
    /// * `module_id` - The module identifier
    /// * `updates` - Map of fields to update
    ///
    /// # Returns
    /// Ok(()) if successful
    pub fn update_credentials(
        &self,
        module_id: &str,
        updates: HashMap<String, String>,
    ) -> CredentialResult<()> {
        // Retrieve existing credentials
        let mut credentials = self.retrieve_credentials(module_id)?;

        // Apply updates
        if let Some(username) = updates.get("username") {
            credentials.username = username.clone();
        }
        if let Some(password) = updates.get("password") {
            credentials.password = password.clone();
        }

        // Update extra fields
        for (key, value) in updates {
            if key != "username" && key != "password" {
                credentials.extra.insert(key, value);
            }
        }

        // Store updated credentials
        self.store_credentials(module_id, &credentials)
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_manager_creation() {
        let manager = AuthManager::new();
        assert!(!manager.has_credentials("nonexistent"));
    }

    #[test]
    fn test_auth_manager_default() {
        let manager = AuthManager::default();
        assert!(!manager.has_credentials("test"));
    }

    // Note: Actual keychain tests require OS keychain services
    // These tests are skipped in CI environments
    #[test]
    #[ignore]
    fn test_store_and_retrieve() {
        let manager = AuthManager::new();
        let module_id = "test_module_auth";

        let mut credentials = AuthCredentials {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            extra: HashMap::new(),
        };
        credentials
            .extra
            .insert("api_key".to_string(), "abc123".to_string());

        // Store
        manager.store_credentials(module_id, &credentials).unwrap();

        // Check exists
        assert!(manager.has_credentials(module_id));

        // Retrieve
        let retrieved = manager.retrieve_credentials(module_id).unwrap();
        assert_eq!(retrieved.username, "testuser");
        assert_eq!(retrieved.password, "testpass");
        assert_eq!(retrieved.extra.get("api_key").unwrap(), "abc123");

        // Clean up
        manager.delete_credentials(module_id).unwrap();
        assert!(!manager.has_credentials(module_id));
    }

    #[test]
    #[ignore]
    fn test_update_credentials() {
        let manager = AuthManager::new();
        let module_id = "test_module_update";

        let credentials = AuthCredentials {
            username: "user1".to_string(),
            password: "pass1".to_string(),
            extra: HashMap::new(),
        };

        // Store initial
        manager.store_credentials(module_id, &credentials).unwrap();

        // Update
        let mut updates = HashMap::new();
        updates.insert("password".to_string(), "newpass".to_string());
        updates.insert("token".to_string(), "xyz789".to_string());

        manager.update_credentials(module_id, updates).unwrap();

        // Verify
        let retrieved = manager.retrieve_credentials(module_id).unwrap();
        assert_eq!(retrieved.username, "user1"); // unchanged
        assert_eq!(retrieved.password, "newpass"); // updated
        assert_eq!(retrieved.extra.get("token").unwrap(), "xyz789"); // added

        // Clean up
        manager.delete_credentials(module_id).unwrap();
    }
}
