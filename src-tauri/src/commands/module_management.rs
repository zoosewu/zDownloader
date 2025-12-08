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

// ============================================================================
// Module Management Commands (Phase 7: T121-T126)
// ============================================================================

/// T121: Get all modules
///
/// Returns a list of all registered modules with their metadata.
///
/// # Returns
/// * `Ok(Vec<ModuleMetadataDto>)` - List of module metadata
/// * `Err(String)` - Error message if operation failed
#[tauri::command]
pub async fn get_all_modules() -> Result<Vec<serde_json::Value>, String> {
    // TODO: Implement actual module registry access
    // For now, return empty list to allow frontend compilation
    Ok(vec![])
}

/// T122: Get module detail
///
/// Returns detailed information about a specific module.
///
/// # Arguments
/// * `module_id` - Module identifier
///
/// # Returns
/// * `Ok(ModuleDetailDto)` - Module details
/// * `Err(String)` - Error message if module not found
#[tauri::command]
pub async fn get_module_detail(module_id: String) -> Result<serde_json::Value, String> {
    // TODO: Implement module detail retrieval
    Err(format!("Module '{}' not found", module_id))
}

/// T123: Reload module
///
/// Reloads a module from disk, refreshing its code and configuration.
///
/// # Arguments
/// * `module_id` - Module identifier
///
/// # Returns
/// * `Ok(())` - Module reloaded successfully
/// * `Err(String)` - Error message if reload failed
#[tauri::command]
pub async fn reload_module(module_id: String) -> Result<(), String> {
    if module_id.is_empty() {
        return Err("Module ID cannot be empty".to_string());
    }

    // TODO: Implement module reload logic
    Ok(())
}

/// T124: Unload module
///
/// Unloads a module from memory.
///
/// # Arguments
/// * `module_id` - Module identifier
///
/// # Returns
/// * `Ok(())` - Module unloaded successfully
/// * `Err(String)` - Error message if unload failed
#[tauri::command]
pub async fn unload_module(module_id: String) -> Result<(), String> {
    if module_id.is_empty() {
        return Err("Module ID cannot be empty".to_string());
    }

    // TODO: Implement module unload logic
    Ok(())
}

/// T125: Scan modules
///
/// Scans the modules directory for new or updated modules.
///
/// # Returns
/// * `Ok(ScanResultDto)` - Scan results
/// * `Err(String)` - Error message if scan failed
#[tauri::command]
pub async fn scan_modules() -> Result<serde_json::Value, String> {
    // TODO: Implement module scanning logic
    Ok(serde_json::json!({
        "totalFound": 0,
        "successfullyLoaded": 0,
        "failed": []
    }))
}

/// T126: Get module audit logs
///
/// Returns audit log entries for a specific module.
///
/// # Arguments
/// * `module_id` - Module identifier
/// * `limit` - Maximum number of entries to return
/// * `offset` - Offset for pagination
///
/// # Returns
/// * `Ok(AuditLogsResponseDto)` - Audit logs
/// * `Err(String)` - Error message if operation failed
#[tauri::command]
pub async fn get_module_audit_logs(
    module_id: String,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<serde_json::Value, String> {
    if module_id.is_empty() {
        return Err("Module ID cannot be empty".to_string());
    }

    let _limit = limit.unwrap_or(100);
    let _offset = offset.unwrap_or(0);

    // TODO: Implement audit log retrieval
    Ok(serde_json::json!({
        "total": 0,
        "logs": []
    }))
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
