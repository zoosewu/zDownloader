// Module System Initialization Helper
//
// This module provides helper functions to initialize and manage the module system

use super::registry::ModuleRegistry;
use std::path::PathBuf;

/// Initialize module system with default settings
///
/// This function:
/// 1. Creates a new ModuleRegistry
/// 2. Scans the default modules directory
/// 3. Loads all valid modules
/// 4. Returns the initialized registry
///
/// # Returns
/// Initialized ModuleRegistry with all modules loaded
///
/// # Example
/// ```ignore
/// let registry = init_module_system();
/// println!("Loaded {} modules", registry.count());
/// ```
pub fn init_module_system() -> ModuleRegistry {
    init_module_system_with_path("./modules")
}

/// Initialize module system with custom path
///
/// # Arguments
/// * `modules_dir` - Path to the modules directory
///
/// # Returns
/// Initialized ModuleRegistry with all modules loaded
///
/// # Example
/// ```ignore
/// let registry = init_module_system_with_path("/custom/modules");
/// ```
pub fn init_module_system_with_path(modules_dir: impl AsRef<std::path::Path>) -> ModuleRegistry {
    log::info!("🔧 Initializing module system...");
    log::info!("📁 Modules directory: {}", modules_dir.as_ref().display());

    let mut registry = ModuleRegistry::new();
    let modules_path = PathBuf::from(modules_dir.as_ref());

    // Check if modules directory exists
    if !modules_path.exists() {
        log::warn!("⚠️  Modules directory not found: {}", modules_path.display());
        log::info!("Creating modules directory...");

        if let Err(e) = std::fs::create_dir_all(&modules_path) {
            log::error!("❌ Failed to create modules directory: {}", e);
            return registry;
        }
    }

    // Scan and load modules
    let loaded_count = registry.scan_and_register(&modules_path);

    log::info!("✅ Module system initialized");
    log::info!("📦 Loaded {} modules", loaded_count);

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
        log::warn!("⚠️  No modules were loaded. Check the modules directory.");
    }

    registry
}

/// Print diagnostic information about the module system
///
/// # Arguments
/// * `registry` - Reference to the ModuleRegistry to diagnose
pub fn diagnose_module_system(registry: &ModuleRegistry) {
    println!("\n========== Module System Diagnostics ==========\n");

    let modules = registry.get_all_js_module_manifests();
    println!("Total modules loaded: {}", modules.len());

    if modules.is_empty() {
        println!("⚠️  No modules loaded!");
        println!("\nTroubleshooting:");
        println!("1. Check if modules directory exists: ./modules");
        println!("2. Verify module structure (manifest.json + index.js)");
        println!("3. Check module permissions");
        println!("4. Review logs for loading errors");
        return;
    }

    println!("\nModules:");
    for (i, manifest) in modules.iter().enumerate() {
        println!("\n{}. {}", i + 1, manifest.name);
        println!("   ID: {}", manifest.id);
        println!("   Version: {}", manifest.version);
        println!("   Author: {}", manifest.author);
        println!("   Description: {}", manifest.description);

        // Show URL patterns if available
        if !manifest.url_patterns.is_empty() {
            println!("   URL Patterns:");
            for pattern in &manifest.url_patterns {
                println!("     - {}", pattern);
            }
        }
    }

    println!("\n===============================================\n");
}

/// Test URL against all loaded modules
///
/// # Arguments
/// * `registry` - Reference to the ModuleRegistry
/// * `url` - URL to test
///
/// # Returns
/// Vector of module names that can handle the URL
pub fn test_url_support(registry: &ModuleRegistry, url: &str) -> Vec<String> {
    log::info!("🔍 Testing URL support: {}", url);

    let modules = registry.get_all_js_module_manifests();
    let mut supported = Vec::new();

    for manifest in modules {
        // Check if any URL pattern matches
        let matches = manifest.url_patterns.iter().any(|pattern| {
            // Simple pattern matching (can be improved)
            if pattern.contains('*') {
                // Wildcard pattern
                let parts: Vec<&str> = pattern.split('*').collect();
                parts.iter().all(|part| url.contains(part))
            } else {
                // Exact match
                url.starts_with(pattern)
            }
        });

        if matches {
            log::info!("  ✓ Supported by: {}", manifest.name);
            supported.push(manifest.name.clone());
        }
    }

    if supported.is_empty() {
        log::info!("  ✗ No module supports this URL");
    }

    supported
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_system_initialization() {
        // Initialize with test directory
        let registry = init_module_system_with_path("../modules");

        // Should not panic
        let modules = registry.get_all_js_module_manifests();
        println!("Loaded {} modules", modules.len());

        // If K2S module exists, it should be loaded
        let k2s_exists = modules.iter().any(|m| m.id == "k2s");
        if k2s_exists {
            println!("✓ K2S module found");
        } else {
            println!("⚠ K2S module not found (may not exist yet)");
        }
    }

    #[test]
    fn test_url_support_detection() {
        let registry = init_module_system_with_path("../modules");

        // Test K2S URL
        let k2s_url = "https://k2s.cc/file/abc123";
        let supported = test_url_support(&registry, k2s_url);

        println!("URL: {}", k2s_url);
        println!("Supported by {} module(s)", supported.len());

        for module in supported {
            println!("  - {}", module);
        }
    }

    #[test]
    fn test_diagnostics() {
        let registry = init_module_system_with_path("../modules");
        diagnose_module_system(&registry);
    }
}
