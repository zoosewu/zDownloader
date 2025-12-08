// Module Management Commands (Phase 6: T108-T111)
//
// Tauri commands for managing module credentials

use crate::modules::credential_proxy::{
    save_credentials, delete_credentials, get_credential_status, CredentialStatus
};
use std::collections::HashMap;
use tauri::State;

/// T108: Save module credentials to secure storage
///
/// Stores credentials for a module in the system keychain.
/// Credentials are stored securely and isolated per module.
///
/// # Arguments
/// * `module_id` - Unique module identifier (e.g., "k2s")
/// * `credentials` - Map of credential keys to values
///
/// # Returns
/// * `Ok(())` - Credentials saved successfully
/// * `Err(String)` - Error message if save failed
///
/// # Example
/// ```javascript
/// await invoke('save_module_credentials', {
///   moduleId: 'k2s',
///   credentials: {
///     username: 'user@example.com',
///     password: 'secret123'
///   }
/// });
/// ```
#[tauri::command]
pub async fn save_module_credentials(
    module_id: String,
    credentials: HashMap<String, String>,
) -> Result<(), String> {
    // T111: Error handling
    if module_id.is_empty() {
        return Err("Module ID cannot be empty".to_string());
    }

    if credentials.is_empty() {
        return Err("Credentials cannot be empty".to_string());
    }

    // Validate credential keys (should not be empty)
    for (key, value) in &credentials {
        if key.is_empty() {
            return Err("Credential key cannot be empty".to_string());
        }
        if value.is_empty() {
            return Err(format!("Credential value for '{}' cannot be empty", key));
        }
    }

    // Save to secure storage
    save_credentials(&module_id, credentials)
        .await
        .map_err(|e| format!("Failed to save credentials: {}", e))
}

/// T109: Get module credentials status
///
/// Checks which credentials are available for a module without
/// exposing the actual credential values.
///
/// # Arguments
/// * `module_id` - Unique module identifier
/// * `required_keys` - List of required credential keys
///
/// # Returns
/// * `Ok(CredentialStatus)` - Status object with available/missing keys
/// * `Err(String)` - Error message if check failed
///
/// # Example
/// ```javascript
/// const status = await invoke('get_module_credentials_status', {
///   moduleId: 'k2s',
///   requiredKeys: ['username', 'password']
/// });
/// // Returns: { moduleId: 'k2s', hasCredentials: true, availableKeys: [...], missingKeys: [...] }
/// ```
#[tauri::command]
pub async fn get_module_credentials_status(
    module_id: String,
    required_keys: Vec<String>,
) -> Result<CredentialStatus, String> {
    // T111: Error handling
    if module_id.is_empty() {
        return Err("Module ID cannot be empty".to_string());
    }

    get_credential_status(&module_id, &required_keys)
        .await
        .map_err(|e| format!("Failed to get credential status: {}", e))
}

/// T110: Delete module credentials
///
/// Removes stored credentials for a module from secure storage.
///
/// # Arguments
/// * `module_id` - Unique module identifier
/// * `keys` - Optional list of specific keys to delete. If empty, deletes all.
///
/// # Returns
/// * `Ok(())` - Credentials deleted successfully
/// * `Err(String)` - Error message if delete failed
///
/// # Example
/// ```javascript
/// // Delete specific credentials
/// await invoke('delete_module_credentials', {
///   moduleId: 'k2s',
///   keys: ['password']
/// });
///
/// // Delete all credentials for the module
/// await invoke('delete_module_credentials', {
///   moduleId: 'k2s',
///   keys: []
/// });
/// ```
#[tauri::command]
pub async fn delete_module_credentials(
    module_id: String,
    keys: Vec<String>,
) -> Result<(), String> {
    // T111: Error handling
    if module_id.is_empty() {
        return Err("Module ID cannot be empty".to_string());
    }

    // If no specific keys provided, we can't delete "all" as we don't know
    // what keys exist. Frontend should provide explicit keys.
    if keys.is_empty() {
        return Err(
            "No credential keys specified for deletion. Please provide the keys to delete."
                .to_string()
        );
    }

    delete_credentials(&module_id, &keys)
        .await
        .map_err(|e| format!("Failed to delete credentials: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test save_module_credentials validation (T108, T111)
    #[tokio::test]
    async fn test_save_credentials_validation() {
        // Empty module ID
        let result = save_module_credentials(
            String::new(),
            HashMap::from([("key".to_string(), "value".to_string())]),
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Module ID cannot be empty"));

        // Empty credentials
        let result = save_module_credentials(
            "test-module".to_string(),
            HashMap::new(),
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Credentials cannot be empty"));

        // Empty credential key
        let mut creds = HashMap::new();
        creds.insert(String::new(), "value".to_string());
        let result = save_module_credentials("test-module".to_string(), creds).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Credential key cannot be empty"));

        // Empty credential value
        let mut creds = HashMap::new();
        creds.insert("key".to_string(), String::new());
        let result = save_module_credentials("test-module".to_string(), creds).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be empty"));
    }

    /// Test get_module_credentials_status validation (T109, T111)
    #[tokio::test]
    async fn test_get_status_validation() {
        // Empty module ID
        let result = get_module_credentials_status(
            String::new(),
            vec!["username".to_string()],
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Module ID cannot be empty"));
    }

    /// Test delete_module_credentials validation (T110, T111)
    #[tokio::test]
    async fn test_delete_credentials_validation() {
        // Empty module ID
        let result = delete_module_credentials(
            String::new(),
            vec!["password".to_string()],
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Module ID cannot be empty"));

        // Empty keys list
        let result = delete_module_credentials(
            "test-module".to_string(),
            vec![],
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No credential keys specified"));
    }
}
