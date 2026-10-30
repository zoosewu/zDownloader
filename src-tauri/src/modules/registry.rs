// Module registry for managing website support modules
use super::loader::ModuleLoader;
use super::js_module::SiteModule;
use super::manifest::ModuleManifest;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::fs;

/// Registry that manages all loaded website modules
pub struct ModuleRegistry {
    /// All registered JavaScript modules (Phase 3: T041)
    js_modules: Arc<RwLock<Vec<SiteModule>>>,
    /// Module loader (kept to keep libraries alive)
    _loader: ModuleLoader,
}

impl ModuleRegistry {
    /// Creates a new empty module registry with default modules directory (./modules)
    pub fn new() -> Self {
        Self::with_path("./modules")
    }

    /// Creates a new empty module registry with custom modules directory
    ///
    /// # Arguments
    /// * `modules_dir` - Path to the modules directory
    pub fn with_path(modules_dir: impl AsRef<Path>) -> Self {
        Self {
            js_modules: Arc::new(RwLock::new(Vec::new())),
            _loader: ModuleLoader::new(modules_dir.as_ref().to_str().unwrap_or("./modules")),
        }
    }

    /// Scans a directory and registers all JavaScript modules found
    ///
    /// Note: This method now loads JavaScript modules instead of native Rust modules.
    /// Native Rust module loading has been replaced by the JavaScript module system (Feature 004).
    ///
    /// # Arguments
    /// * `_dir` - Directory containing module subdirectories (currently ignored, uses default loader path)
    ///
    /// # Returns
    /// Number of modules successfully loaded and registered
    pub fn scan_and_register(&mut self, _dir: impl AsRef<Path>) -> usize {
        // Note: The new ModuleLoader loads JavaScript modules from its configured directory
        // The dir parameter is ignored since loader was configured at creation time
        let new_modules = self._loader.load_all_modules();
        let count = new_modules.len();

        // Register as JavaScript modules instead of Rust modules
        let mut js_modules = self.js_modules.write().unwrap();
        js_modules.extend(new_modules);

        count
    }

    /// Scans and registers JavaScript modules from a directory (Phase 3: T043-T044)
    ///
    /// # Arguments
    /// * `modules_dir` - Directory containing module subdirectories
    ///
    /// # Returns
    /// Number of JavaScript modules successfully loaded
    pub fn scan_and_register_js_modules(&self, modules_dir: impl AsRef<Path>) -> Result<usize, String> {
        let modules_path = modules_dir.as_ref();

        if !modules_path.exists() {
            return Err(format!("Modules directory does not exist: {:?}", modules_path));
        }

        let mut count = 0;
        let entries = fs::read_dir(modules_path)
            .map_err(|e| format!("Failed to read modules directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            // Skip if not a directory
            if !path.is_dir() {
                continue;
            }

            // Try to load module from this directory
            match self.load_js_module_from_dir(&path) {
                Ok(_) => count += 1,
                Err(e) => {
                    // Log error but continue with other modules
                    eprintln!("Failed to load module from {:?}: {}", path, e);
                }
            }
        }

        Ok(count)
    }

    /// Loads a JavaScript module from a directory (Phase 3: T044)
    ///
    /// # Arguments
    /// * `module_dir` - Directory containing manifest.json and index.js
    fn load_js_module_from_dir(&self, module_dir: &Path) -> Result<(), String> {
        // Read manifest.json
        let manifest_path = module_dir.join("manifest.json");
        let manifest_json = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest.json: {}", e))?;

        let manifest = ModuleManifest::from_json(&manifest_json)?;

        // Read index.js
        let code_path = module_dir.join("index.js");
        let code = fs::read_to_string(&code_path)
            .map_err(|e| format!("Failed to read index.js: {}", e))?;

        // Create module instance
        let module = SiteModule::new(manifest.id.clone(), manifest, code);

        // Register module
        let mut js_modules = self.js_modules.write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;
        js_modules.push(module);

        Ok(())
    }

    /// Registers a JavaScript module (Phase 3: T041)
    pub fn register_js_module(&self, module: SiteModule) -> Result<(), String> {
        let mut js_modules = self.js_modules.write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;
        js_modules.push(module);
        Ok(())
    }

    /// Gets count of JavaScript modules
    pub fn js_module_count(&self) -> usize {
        self.js_modules.read().unwrap().len()
    }

    /// Finds a JavaScript module for the given URL (Phase 3: T045)
    ///
    /// Returns the first JS module that can handle the URL
    /// If multiple modules match, returns the first one (T046: conflict resolution)
    pub fn find_js_module_for_url(&self, url: &str) -> Option<String> {
        let js_modules = self.js_modules.read().unwrap();

        for module in js_modules.iter() {
            // Check if URL matches any pattern in the manifest
            for pattern in &module.manifest.url_patterns {
                if Self::url_matches_pattern(url, pattern) {
                    return Some(module.id.clone());
                }
            }
        }

        None
    }

    /// Checks if a URL matches a pattern (wildcard support)
    fn url_matches_pattern(url: &str, pattern: &str) -> bool {
        // Simple pattern matching with * wildcard
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let (prefix, suffix) = (parts[0], parts[1]);
                return url.starts_with(prefix) && url.ends_with(suffix);
            }
        }

        // Exact match
        url == pattern || url.starts_with(pattern)
    }

    /// Executes a function with mutable access to a JavaScript module
    ///
    /// # Arguments
    /// * `module_id` - The module ID to access
    /// * `f` - Function to execute with mutable module reference
    ///
    /// # Returns
    /// Result from the function, or error if module not found
    pub fn with_js_module_mut<F, R>(&self, module_id: &str, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut SiteModule) -> R,
    {
        let mut js_modules = self.js_modules.write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;

        let module = js_modules.iter_mut()
            .find(|m| m.id == module_id)
            .ok_or_else(|| format!("Module not found: {}", module_id))?;

        Ok(f(module))
    }

    /// Clears all JavaScript modules
    pub fn clear_js_modules(&self) {
        let mut js_modules = self.js_modules.write().unwrap();
        js_modules.clear();
    }

    /// Gets all JavaScript module manifests
    ///
    /// # Returns
    /// Vector of module manifests with their metadata
    pub fn get_all_js_module_manifests(&self) -> Vec<ModuleManifest> {
        let js_modules = self.js_modules.read().unwrap();
        js_modules.iter().map(|m| m.manifest.clone()).collect()
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_scan_nonexistent_directory() {
        let mut registry = ModuleRegistry::new();
        let count = registry.scan_and_register("/nonexistent/directory");
        assert_eq!(count, 0);
    }

    // Phase 3: T048 - Module registry tests for JavaScript modules
    #[test]
    fn test_js_module_registration() {
        use crate::modules::js_module::SiteModule;
        use crate::modules::manifest::{ModuleManifest, ModulePermissions, NetworkPermission};

        let registry = ModuleRegistry::new();

        let manifest = ModuleManifest {
            id: "test-js".to_string(),
            name: "Test JS Module".to_string(),
            version: "1.0.0".to_string(),
            author: "Test".to_string(),
            description: "Test".to_string(),
            homepage: None,
            url_patterns: vec!["https://test.com/*".to_string()],
            permissions: ModulePermissions {
                network: NetworkPermission {
                    allow_domains: vec!["test.com".to_string()],
                },
                credentials: None,
            },
            entrypoint: "index.js".to_string(),
            class_name: None,
        };

        let code = "class TestJs { canHandle(url) { return true; } }".to_string();
        let module = SiteModule::new("test-js".to_string(), manifest, code);

        let result = registry.register_js_module(module);
        assert!(result.is_ok());
        assert_eq!(registry.js_module_count(), 1);
    }

    #[test]
    fn test_find_js_module_for_url() {
        use crate::modules::js_module::SiteModule;
        use crate::modules::manifest::{ModuleManifest, ModulePermissions, NetworkPermission};

        let registry = ModuleRegistry::new();

        let manifest = ModuleManifest {
            id: "test-js".to_string(),
            name: "Test JS Module".to_string(),
            version: "1.0.0".to_string(),
            author: "Test".to_string(),
            description: "Test".to_string(),
            homepage: None,
            url_patterns: vec!["https://example.com/file/*".to_string()],
            permissions: ModulePermissions {
                network: NetworkPermission {
                    allow_domains: vec!["example.com".to_string()],
                },
                credentials: None,
            },
            entrypoint: "index.js".to_string(),
            class_name: None,
        };

        let code = "class TestJs {}".to_string();
        let module = SiteModule::new("test-js".to_string(), manifest, code);
        registry.register_js_module(module).unwrap();

        // Should find the module
        let result = registry.find_js_module_for_url("https://example.com/file/test.zip");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "test-js");

        // Should not find for different URL
        let result = registry.find_js_module_for_url("https://other.com/file.zip");
        assert!(result.is_none());
    }

    #[test]
    fn test_url_pattern_matching() {
        // Test exact match
        assert!(ModuleRegistry::url_matches_pattern(
            "https://example.com/file.zip",
            "https://example.com/file.zip"
        ));

        // Test prefix match
        assert!(ModuleRegistry::url_matches_pattern(
            "https://example.com/file.zip",
            "https://example.com/"
        ));

        // Test wildcard match
        assert!(ModuleRegistry::url_matches_pattern(
            "https://example.com/path/file.zip",
            "https://example.com/*.zip"
        ));

        // Test no match
        assert!(!ModuleRegistry::url_matches_pattern(
            "https://other.com/file.zip",
            "https://example.com/*"
        ));
    }

    #[test]
    fn test_clear_js_modules() {
        use crate::modules::js_module::SiteModule;
        use crate::modules::manifest::{ModuleManifest, ModulePermissions, NetworkPermission};

        let registry = ModuleRegistry::new();

        let manifest = ModuleManifest {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            author: "Test".to_string(),
            description: "Test".to_string(),
            homepage: None,
            url_patterns: vec!["https://test.com/*".to_string()],
            permissions: ModulePermissions {
                network: NetworkPermission {
                    allow_domains: vec!["test.com".to_string()],
                },
                credentials: None,
            },
            entrypoint: "index.js".to_string(),
            class_name: None,
        };

        let module = SiteModule::new("test".to_string(), manifest, "class Test {}".to_string());
        registry.register_js_module(module).unwrap();
        assert_eq!(registry.js_module_count(), 1);

        registry.clear_js_modules();
        assert_eq!(registry.js_module_count(), 0);
    }
}
#[cfg(test)]
mod registry_drop_tests {
    use crate::modules::registry::ModuleRegistry;

    #[test]
    fn test_registry_can_be_dropped_safely() {
        // This test ensures that ModuleRegistry can be created, used, and dropped
        // without V8 HandleScope errors
        
        let modules_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../modules");
        let mut registry = ModuleRegistry::with_path(modules_dir);
        
        // Scan and load modules
        registry.scan_and_register(modules_dir);
        
        // Get manifests (this was causing the error)
        let manifests = registry.get_all_js_module_manifests();
        
        println!("✅ Successfully loaded {} modules", manifests.len());
        println!("✅ Registry can be dropped safely");
        
        // Registry will be dropped here - this should not crash
    }
}
