// Module Management Commands (Phase 6: T108-T111)
//
// Tauri commands for managing module credentials

use crate::modules::credential_proxy::{
    save_credentials, delete_credentials, get_credential_status, CredentialStatus
};
use std::collections::HashMap;
use std::path::PathBuf;

/// Find the modules directory by trying multiple possible locations
///
/// This function uses different search strategies based on build mode:
/// - **Release mode**: Only uses ./modules for optimal performance
/// - **Debug/Dev mode**: Smart search across multiple locations
///
/// Debug mode search order:
/// 1. ./modules (current directory)
/// 2. ../../modules (from target/debug/)
/// 3. ../modules (from src-tauri/)
/// 4. Look for project root by finding Cargo.toml
fn find_modules_directory() -> PathBuf {
    // Release mode: only use ./modules for performance
    // This is compiled out in debug builds (zero overhead)
    #[cfg(not(debug_assertions))]
    {
        log::info!("🚀 Release mode: using ./modules");
        return PathBuf::from("./modules");
    }

    // Debug/Dev mode: smart search for developer convenience
    // This entire block is compiled out in release builds (zero overhead)
    #[cfg(debug_assertions)]
    {
        log::info!("🔧 Debug mode: searching for modules directory...");

        let candidate_paths = vec![
            PathBuf::from("./modules"),           // Current directory
            PathBuf::from("../../modules"),       // From target/debug
            PathBuf::from("../modules"),          // From src-tauri
            PathBuf::from("../../../modules"),    // From deep nested directories
        ];

        // Try each candidate path
        for path in &candidate_paths {
            if path.exists() && path.is_dir() {
                log::info!("✓ Found modules directory at: {:?}", path.canonicalize().unwrap_or(path.clone()));
                return path.clone();
            }
        }

        // Try to find project root by looking for Cargo.toml
        if let Ok(current_exe) = std::env::current_exe() {
            let mut search_dir = current_exe.parent();

            // Search up to 5 levels
            for _ in 0..5 {
                if let Some(dir) = search_dir {
                    let cargo_toml = dir.join("Cargo.toml");
                    let modules_dir = dir.join("modules");

                    if cargo_toml.exists() && modules_dir.exists() {
                        log::info!("✓ Found modules directory via Cargo.toml at: {:?}",
                            modules_dir.canonicalize().unwrap_or(modules_dir.clone()));
                        return modules_dir;
                    }

                    search_dir = dir.parent();
                } else {
                    break;
                }
            }
        }

        // Fallback: use ./modules (will be created if needed)
        let fallback = PathBuf::from("./modules");
        log::warn!("⚠️  Could not find existing modules directory, using fallback: {:?}",
            std::env::current_dir().unwrap_or_default().join(&fallback));
        fallback
    }
}

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
pub fn get_all_modules() -> Result<Vec<serde_json::Value>, String> {
    log::info!("📋 Getting all modules...");

    // Find modules directory using smart path resolution
    let modules_dir = find_modules_directory();

    // Create a new registry with the found modules directory
    // Note: This is a workaround since ModuleRegistry cannot be stored in Tauri State
    // due to Deno runtime not being Send + Sync
    let mut registry = crate::modules::registry::ModuleRegistry::with_path(&modules_dir);

    if modules_dir.exists() {
        registry.scan_and_register(&modules_dir);
    }

    let manifests = registry.get_all_js_module_manifests();

    log::info!("✅ Found {} module(s)", manifests.len());

    // Convert manifests to JSON values for frontend
    let mut result = Vec::new();
    for manifest in manifests {
        let json = serde_json::to_value(&manifest)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
        result.push(json);
    }

    Ok(result)
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
pub fn scan_modules() -> Result<serde_json::Value, String> {
    log::info!("🔍 Scanning modules directory...");

    let modules_dir = find_modules_directory();

    // Check if directory exists
    if !modules_dir.exists() {
        log::warn!("⚠️  Modules directory does not exist: {:?}", modules_dir);
        log::info!("Creating modules directory...");

        std::fs::create_dir_all(&modules_dir)
            .map_err(|e| format!("Failed to create modules directory: {}", e))?;

        return Ok(serde_json::json!({
            "totalFound": 0,
            "successfullyLoaded": 0,
            "failed": []
        }));
    }

    log::info!("📁 Scanning directory: {:?}", modules_dir.canonicalize().unwrap_or(modules_dir.clone()));

    // Create a new registry with the found modules directory and scan modules
    // Note: This is a workaround since ModuleRegistry cannot be stored in Tauri State
    let mut registry = crate::modules::registry::ModuleRegistry::with_path(&modules_dir);
    let loaded_count = registry.scan_and_register(&modules_dir);

    log::info!("✅ Scan complete");
    log::info!("📦 Successfully loaded {} module(s)", loaded_count);

    // List loaded modules
    if loaded_count > 0 {
        log::info!("Loaded modules:");
        for manifest in registry.get_all_js_module_manifests() {
            log::info!(
                "  • {} v{} (ID: {})",
                manifest.name,
                manifest.version,
                manifest.id
            );
        }
    } else {
        log::warn!("⚠️  No modules found in directory");
        log::info!("Troubleshooting:");
        log::info!("1. Check if modules directory contains module folders");
        log::info!("2. Each module needs manifest.json and index.js");
        log::info!("3. Verify module permissions");
    }

    Ok(serde_json::json!({
        "totalFound": loaded_count,
        "successfullyLoaded": loaded_count,
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
