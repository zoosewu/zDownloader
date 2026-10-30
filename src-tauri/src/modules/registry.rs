// Module registry for managing website support modules
#![allow(dead_code)]
use super::interface::{BoxedModule, ModuleMetadata};
use super::loader::ModuleLoader;
use super::js_module::SiteModule;
use super::manifest::ModuleManifest;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::fs;

/// Type of module (Phase 3: T042)
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleType {
    /// Native Rust module (dynamic library)
    Rust,
    /// JavaScript module
    JavaScript,
}

/// Registry that manages all loaded website modules
pub struct ModuleRegistry {
    /// All registered Rust modules
    modules: Arc<RwLock<Vec<BoxedModule>>>,
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
            modules: Arc::new(RwLock::new(Vec::new())),
            js_modules: Arc::new(RwLock::new(Vec::new())),
            _loader: ModuleLoader::new(modules_dir.as_ref().to_str().unwrap_or("./modules")),
        }
    }

    /// Registers a module
    ///
    /// # Arguments
    /// * `module` - The module to register
    pub fn register(&self, module: BoxedModule) {
        let mut modules = self.modules.write().unwrap();
        modules.push(module);
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

    /// Finds a module that can handle the given URL
    ///
    /// # Arguments
    /// * `url` - The URL to check
    ///
    /// # Returns
    /// Reference to the first module that can handle the URL,
    /// or None if no module can handle it
    ///
    /// # Note
    /// If multiple modules can handle the URL, the first one
    /// registered is returned.
    pub fn find_module_for_url(&self, url: &str) -> Option<Arc<RwLock<Vec<BoxedModule>>>> {
        let modules = self.modules.read().unwrap();

        for module in modules.iter() {
            if module.can_handle(url) {
                drop(modules); // Release read lock
                return Some(Arc::clone(&self.modules));
            }
        }

        None
    }

    /// Gets metadata for all registered modules
    ///
    /// # Returns
    /// Vector of module metadata
    pub fn get_all_metadata(&self) -> Vec<ModuleMetadata> {
        let modules = self.modules.read().unwrap();
        modules.iter().map(|m| m.metadata()).collect()
    }

    /// Gets metadata for a specific module by ID
    ///
    /// # Arguments
    /// * `module_id` - The module ID to search for
    ///
    /// # Returns
    /// Module metadata if found
    pub fn get_module_metadata(&self, module_id: &str) -> Option<ModuleMetadata> {
        let modules = self.modules.read().unwrap();

        for module in modules.iter() {
            let metadata = module.metadata();
            if metadata.id == module_id {
                return Some(metadata);
            }
        }

        None
    }

    /// Checks if any module can handle the given URL
    ///
    /// # Arguments
    /// * `url` - The URL to check
    ///
    /// # Returns
    /// `true` if at least one module can handle the URL
    pub fn can_handle_url(&self, url: &str) -> bool {
        let modules = self.modules.read().unwrap();
        modules.iter().any(|m| m.can_handle(url))
    }

    /// Returns the number of registered modules
    pub fn count(&self) -> usize {
        let modules = self.modules.read().unwrap();
        modules.len()
    }

    /// Clears all registered modules
    ///
    /// # Note
    /// This will drop all modules and potentially unload their libraries
    pub fn clear(&self) {
        let mut modules = self.modules.write().unwrap();
        modules.clear();
    }

    /// Gets a list of all module IDs
    pub fn get_module_ids(&self) -> Vec<String> {
        let modules = self.modules.read().unwrap();
        modules.iter().map(|m| m.metadata().id).collect()
    }

    /// Provides read-only access to modules for complex operations
    ///
    /// # Returns
    /// Arc to the modules vector
    pub fn modules(&self) -> Arc<RwLock<Vec<BoxedModule>>> {
        Arc::clone(&self.modules)
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

    /// Gets total module count (Rust + JavaScript)
    pub fn total_module_count(&self) -> usize {
        self.count() + self.js_module_count()
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

    /// Finds best module for URL considering both Rust and JS modules (Phase 3: T045-T047)
    ///
    /// Priority order:
    /// 1. JavaScript modules (checked first as they're more specific)
    /// 2. Rust modules
    /// 3. None if no module can handle (fallback to generic downloader - T047)
    ///
    /// # Returns
    /// - `Some(("js", module_id))` if a JS module can handle
    /// - `Some(("rust", module_id))` if a Rust module can handle
    /// - `None` if no module can handle (should fallback to generic downloader)
    pub fn find_best_module_for_url(&self, url: &str) -> Option<(ModuleType, String)> {
        // Phase 3: T045 - URL matching logic
        // Try JavaScript modules first
        if let Some(js_id) = self.find_js_module_for_url(url) {
            return Some((ModuleType::JavaScript, js_id));
        }

        // Try Rust modules
        if self.can_handle_url(url) {
            // Find the specific Rust module
            let modules = self.modules.read().unwrap();
            for module in modules.iter() {
                if module.can_handle(url) {
                    return Some((ModuleType::Rust, module.metadata().id));
                }
            }
        }

        // Phase 3: T047 - Fallback to generic downloader
        None
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
    use crate::modules::interface::{AuthCredentials, CaptchaChallenge, CaptchaSolution, DownloadInfo, ModuleResult, WebsiteModule};
    use std::collections::HashMap;

    // Mock module for testing
    struct MockModule {
        id: String,
        can_handle_urls: Vec<String>,
    }

    impl MockModule {
        fn new(id: &str, urls: Vec<&str>) -> Self {
            Self {
                id: id.to_string(),
                can_handle_urls: urls.iter().map(|s| s.to_string()).collect(),
            }
        }
    }

    impl WebsiteModule for MockModule {
        fn metadata(&self) -> ModuleMetadata {
            ModuleMetadata {
                id: self.id.clone(),
                name: format!("Mock Module {}", self.id),
                version: "1.0.0".to_string(),
                author: "Test".to_string(),
                url_patterns: self.can_handle_urls.clone(),
                requires_auth: false,
                supports_captcha: false,
            }
        }

        fn can_handle(&self, url: &str) -> bool {
            self.can_handle_urls.iter().any(|pattern| url.contains(pattern))
        }

        fn extract_download_info(
            &self,
            _url: &str,
            _credentials: Option<&AuthCredentials>,
        ) -> ModuleResult<DownloadInfo> {
            Ok(DownloadInfo {
                download_url: "http://example.com/file.zip".to_string(),
                filename: Some("file.zip".to_string()),
                file_size: Some(1024),
                headers: HashMap::new(),
                requires_auth: false,
            })
        }

        fn handle_captcha(
            &self,
            _challenge: &CaptchaChallenge,
            _solution: &CaptchaSolution,
        ) -> ModuleResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = ModuleRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_register_module() {
        let registry = ModuleRegistry::new();
        let module = Box::new(MockModule::new("test", vec!["example.com"]));

        registry.register(module);
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_can_handle_url() {
        let registry = ModuleRegistry::new();
        let module = Box::new(MockModule::new("test", vec!["example.com"]));

        registry.register(module);

        assert!(registry.can_handle_url("http://example.com/file"));
        assert!(!registry.can_handle_url("http://other.com/file"));
    }

    #[test]
    fn test_get_metadata() {
        let registry = ModuleRegistry::new();
        let module = Box::new(MockModule::new("test-module", vec!["example.com"]));

        registry.register(module);

        let metadata = registry.get_module_metadata("test-module");
        assert!(metadata.is_some());

        let metadata = metadata.unwrap();
        assert_eq!(metadata.id, "test-module");
        assert_eq!(metadata.name, "Mock Module test-module");
    }

    #[test]
    fn test_clear_modules() {
        let registry = ModuleRegistry::new();
        registry.register(Box::new(MockModule::new("test1", vec!["example.com"])));
        registry.register(Box::new(MockModule::new("test2", vec!["other.com"])));

        assert_eq!(registry.count(), 2);

        registry.clear();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_get_module_ids() {
        let registry = ModuleRegistry::new();
        registry.register(Box::new(MockModule::new("module1", vec!["example.com"])));
        registry.register(Box::new(MockModule::new("module2", vec!["other.com"])));

        let ids = registry.get_module_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"module1".to_string()));
        assert!(ids.contains(&"module2".to_string()));
    }

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
    fn test_find_best_module_priority() {
        use crate::modules::js_module::SiteModule;
        use crate::modules::manifest::{ModuleManifest, ModulePermissions, NetworkPermission};

        let registry = ModuleRegistry::new();

        // Register a JS module
        let manifest = ModuleManifest {
            id: "js-module".to_string(),
            name: "JS Module".to_string(),
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

        let code = "class JsModule {}".to_string();
        let module = SiteModule::new("js-module".to_string(), manifest, code);
        registry.register_js_module(module).unwrap();

        // JS module should be found first (priority)
        let result = registry.find_best_module_for_url("https://test.com/file.zip");
        assert!(result.is_some());
        let (module_type, id) = result.unwrap();
        assert_eq!(module_type, ModuleType::JavaScript);
        assert_eq!(id, "js-module");
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
    use crate::modules::loader::ModuleLoader;
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
