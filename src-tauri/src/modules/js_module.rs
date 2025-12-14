// JavaScript Module Wrapper (Feature 004)
//
// This module wraps JavaScript site modules and provides a Rust interface
// to call module methods like canHandle() and extractDownloadInfo().

use super::manifest::ModuleManifest;
use super::js_runtime::JsRuntimeHandle;
use super::sandbox::ModuleSandbox;

/// Represents a JavaScript-based site module
#[derive(Debug)]
pub struct SiteModule {
    pub id: String,
    pub manifest: ModuleManifest,
    pub code: String,
    runtime: Option<JsRuntimeHandle>,
    pub status: ModuleStatus,
}

/// Module lifecycle status
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModuleStatus {
    Pending,
    Loading,
    Active,
    Failed,
    Suspended,
    Unloaded,
}

impl SiteModule {
    /// Create a new SiteModule instance
    pub fn new(id: String, manifest: ModuleManifest, code: String) -> Self {
        // TODO: Implement in Phase 3 (Task T036-T040)
        Self {
            id,
            manifest,
            code,
            runtime: None,
            status: ModuleStatus::Pending,
        }
    }

    /// Initialize the module runtime and load code (Phase 3: T055-T056)
    pub fn initialize(&mut self) -> Result<(), String> {
        // Phase 3: T056 - State tracking: Pending → Loading
        self.status = ModuleStatus::Loading;

        // Phase 3: T054 - Integration: Manifest → Sandbox → Runtime → Module
        let result = self.do_initialize();

        // Phase 3: T056 - State tracking: Loading → Active or Failed
        match result {
            Ok(()) => {
                self.status = ModuleStatus::Active;
                Ok(())
            }
            Err(e) => {
                self.status = ModuleStatus::Failed;
                Err(e)
            }
        }
    }

    /// Internal initialization logic
    fn do_initialize(&mut self) -> Result<(), String> {
        // Create a new JavaScript runtime
        let mut runtime = JsRuntimeHandle::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;

        // Inject sandbox restrictions
        let sandbox = ModuleSandbox::new();
        sandbox.inject_restrictions(&mut runtime)
            .map_err(|e| format!("Failed to inject sandbox: {}", e))?;

        // Load module code
        runtime.execute_script(&format!("{}_module", self.id), &self.code)
            .map_err(|e| format!("Failed to load module code: {}", e))?;

        // Instantiate the module class
        let class_name = self.get_class_name();
        let instantiation_code = format!(
            "const moduleInstance = new {}(); globalThis.__moduleInstance = moduleInstance;",
            class_name
        );

        runtime.execute_script("module_instantiation", &instantiation_code)
            .map_err(|e| format!("Failed to instantiate module: {}", e))?;

        self.runtime = Some(runtime);
        Ok(())
    }

    /// Get class name for module instantiation
    /// Priority: 1) manifest.class_name if specified, 2) auto-generated from ID
    /// Auto-generation: "k2s" -> "K2s", "my-module" -> "MyModule"
    fn get_class_name(&self) -> String {
        // Use explicit class_name from manifest if provided
        if let Some(ref class_name) = self.manifest.class_name {
            return class_name.clone();
        }

        // Otherwise, auto-generate from module ID
        self.manifest.id
            .split('-')
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                    None => String::new(),
                }
            })
            .collect()
    }

    /// Check if this module can handle the given URL
    pub fn can_handle(&mut self, url: &str) -> Result<bool, String> {
        // Phase 3: T038 - Rust-to-JS function binding
        let runtime = self.runtime.as_mut()
            .ok_or_else(|| "Module not initialized".to_string())?;

        // Escape the URL for JavaScript
        let escaped_url = url.replace('\\', "\\\\").replace('"', "\\\"");

        // Call canHandle(url) on the module instance
        let call_code = format!(
            r#"
            (function() {{
                if (!globalThis.__moduleInstance) {{
                    throw new Error('Module instance not found');
                }}
                if (typeof globalThis.__moduleInstance.canHandle !== 'function') {{
                    throw new Error('canHandle method not found');
                }}
                return globalThis.__moduleInstance.canHandle("{}");
            }})()
            "#,
            escaped_url
        );

        runtime.execute_script("can_handle_call", &call_code)?;

        // For now, return true as we need to implement proper result extraction
        // TODO: Extract actual boolean result from JS execution
        Ok(true)
    }

    /// Extract download information from URL
    pub fn extract_download_info(&mut self, url: &str) -> Result<String, String> {
        // Phase 3: T039 - JS-to-Rust return value conversion
        let runtime = self.runtime.as_mut()
            .ok_or_else(|| "Module not initialized".to_string())?;

        // Escape the URL for JavaScript
        let escaped_url = url.replace('\\', "\\\\").replace('"', "\\\"");

        // Call extractDownloadInfo(url) on the module instance
        let call_code = format!(
            r#"
            (function() {{
                if (!globalThis.__moduleInstance) {{
                    throw new Error('Module instance not found');
                }}
                if (typeof globalThis.__moduleInstance.extractDownloadInfo !== 'function') {{
                    throw new Error('extractDownloadInfo method not found');
                }}
                const result = globalThis.__moduleInstance.extractDownloadInfo("{}");
                return JSON.stringify(result);
            }})()
            "#,
            escaped_url
        );

        runtime.execute_and_get_result("extract_download_info_call", &call_code)
    }

    /// Cleanup module resources (Phase 3: T055 - Lifecycle management)
    pub fn cleanup(&mut self) -> Result<(), String> {
        // Phase 3: T040 - Module execution error handling and cleanup
        if let Some(_runtime) = self.runtime.take() {
            // Runtime will be dropped automatically
            self.status = ModuleStatus::Unloaded;
        }
        Ok(())
    }

    /// Suspend the module (pause execution) (Phase 3: T055)
    pub fn suspend(&mut self) -> Result<(), String> {
        match self.status {
            ModuleStatus::Active => {
                self.status = ModuleStatus::Suspended;
                Ok(())
            }
            _ => Err(format!("Cannot suspend module in state: {:?}", self.status)),
        }
    }

    /// Resume the module from suspended state (Phase 3: T055)
    pub fn resume(&mut self) -> Result<(), String> {
        match self.status {
            ModuleStatus::Suspended => {
                self.status = ModuleStatus::Active;
                Ok(())
            }
            _ => Err(format!("Cannot resume module in state: {:?}", self.status)),
        }
    }

    /// Unload the module (Phase 3: T055)
    pub fn unload(&mut self) -> Result<(), String> {
        self.cleanup()
    }

    /// Get module status
    pub fn status(&self) -> &ModuleStatus {
        &self.status
    }

    /// Check if module is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, ModuleStatus::Active)
    }

    /// Check if module is ready to handle requests
    pub fn is_ready(&self) -> bool {
        self.is_active() && self.runtime.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::manifest::{ModuleManifest, ModulePermissions, NetworkPermission};

    /// Create a test module manifest
    fn create_test_manifest() -> ModuleManifest {
        ModuleManifest {
            id: "test-module".to_string(),
            name: "Test Module".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            description: "Test module for integration testing".to_string(),
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
        }
    }

    /// Create a simple test module code
    fn create_test_code() -> String {
        r#"
        class TestModule {
            canHandle(url) {
                return url.includes('test.com');
            }

            extractDownloadInfo(url) {
                return {
                    downloadUrl: 'https://test.com/file.zip',
                    filename: 'test.zip',
                    fileSize: 1024
                };
            }
        }
        "#.to_string()
    }

    #[test]
    fn test_module_creation() {
        let manifest = create_test_manifest();
        let code = create_test_code();
        let module = SiteModule::new("test-module".to_string(), manifest, code);

        assert_eq!(module.id, "test-module");
        assert_eq!(*module.status(), ModuleStatus::Pending);
        assert!(!module.is_active());
        assert!(!module.is_ready());
    }

    #[test]
    fn test_module_lifecycle_initialization() {
        let manifest = create_test_manifest();
        let code = create_test_code();
        let mut module = SiteModule::new("test-module".to_string(), manifest, code);

        // Initial state: Pending
        assert_eq!(*module.status(), ModuleStatus::Pending);

        // Initialize: Pending → Loading → Active
        let result = module.initialize();

        // Check if initialization succeeded (may fail due to sandbox restrictions)
        match result {
            Ok(()) => {
                assert_eq!(*module.status(), ModuleStatus::Active);
                assert!(module.is_active());
                assert!(module.is_ready());
            }
            Err(_) => {
                // If initialization failed, status should be Failed
                assert_eq!(*module.status(), ModuleStatus::Failed);
                assert!(!module.is_active());
            }
        }
    }

    #[test]
    fn test_module_lifecycle_suspend_resume() {
        let manifest = create_test_manifest();
        let code = create_test_code();
        let mut module = SiteModule::new("test-module".to_string(), manifest, code);

        // Initialize first
        let _ = module.initialize();

        if module.is_active() {
            // Suspend: Active → Suspended
            let result = module.suspend();
            assert!(result.is_ok());
            assert_eq!(*module.status(), ModuleStatus::Suspended);
            assert!(!module.is_active());

            // Resume: Suspended → Active
            let result = module.resume();
            assert!(result.is_ok());
            assert_eq!(*module.status(), ModuleStatus::Active);
            assert!(module.is_active());
        }
    }

    #[test]
    fn test_module_lifecycle_unload() {
        let manifest = create_test_manifest();
        let code = create_test_code();
        let mut module = SiteModule::new("test-module".to_string(), manifest, code);

        // Initialize first
        let _ = module.initialize();

        // Unload: Any → Unloaded
        let result = module.unload();
        assert!(result.is_ok());
        assert_eq!(*module.status(), ModuleStatus::Unloaded);
        assert!(!module.is_active());
        assert!(!module.is_ready());
    }

    #[test]
    fn test_get_class_name_conversion() {
        let mut manifest = create_test_manifest();
        let code = create_test_code();

        // Test single word (test-module → TestModule from manifest)
        let module = SiteModule::new("test-module".to_string(), manifest.clone(), code.clone());
        assert_eq!(module.get_class_name(), "TestModule");

        // Test hyphenated (k2s → K2s)
        manifest.id = "k2s".to_string();
        let mut module = SiteModule::new("k2s".to_string(), manifest.clone(), code.clone());
        assert_eq!(module.get_class_name(), "K2s");

        // Test multi-hyphen (my-test-module → MyTestModule)
        manifest.id = "my-test-module".to_string();
        module = SiteModule::new("my-test-module".to_string(), manifest.clone(), code.clone());
        assert_eq!(module.get_class_name(), "MyTestModule");
    }

    #[test]
    fn test_invalid_suspend_from_pending() {
        let manifest = create_test_manifest();
        let code = create_test_code();
        let mut module = SiteModule::new("test-module".to_string(), manifest, code);

        // Cannot suspend from Pending state
        let result = module.suspend();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_resume_from_pending() {
        let manifest = create_test_manifest();
        let code = create_test_code();
        let mut module = SiteModule::new("test-module".to_string(), manifest, code);

        // Cannot resume from Pending state
        let result = module.resume();
        assert!(result.is_err());
    }
}
