// Module registry for managing website support modules
use super::interface::{BoxedModule, ModuleMetadata, WebsiteModule};
use super::loader::ModuleLoader;
use std::path::Path;
use std::sync::{Arc, RwLock};

/// Registry that manages all loaded website modules
pub struct ModuleRegistry {
    /// All registered modules
    modules: Arc<RwLock<Vec<BoxedModule>>>,
    /// Module loader (kept to keep libraries alive)
    _loader: ModuleLoader,
}

impl ModuleRegistry {
    /// Creates a new empty module registry
    pub fn new() -> Self {
        Self {
            modules: Arc::new(RwLock::new(Vec::new())),
            _loader: ModuleLoader::new(),
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

    /// Scans a directory and registers all modules found
    ///
    /// # Arguments
    /// * `dir` - Directory containing module libraries
    ///
    /// # Returns
    /// Number of modules successfully loaded and registered
    pub fn scan_and_register(&mut self, dir: impl AsRef<Path>) -> usize {
        let new_modules = self._loader.load_modules_from_dir(dir);
        let count = new_modules.len();

        let mut modules = self.modules.write().unwrap();
        modules.extend(new_modules);

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
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::interface::{AuthCredentials, CaptchaChallenge, CaptchaSolution, DownloadInfo, ModuleResult};
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
}
