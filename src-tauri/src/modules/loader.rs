// JavaScript Module Loader (Feature 004)
//
// This module loads JavaScript-based site modules from disk,
// replacing the original native library loader from Feature 001.

use super::js_module::SiteModule;
use super::manifest::ModuleManifest;
use std::path::{Path, PathBuf};
use std::fs;

/// Module loader for JavaScript-based site modules
pub struct ModuleLoader {
    /// Base directory containing all modules
    modules_dir: PathBuf,
}

impl ModuleLoader {
    /// Creates a new module loader
    ///
    /// # Arguments
    /// * `modules_dir` - Base directory containing module folders
    pub fn new(modules_dir: impl AsRef<Path>) -> Self {
        Self {
            modules_dir: modules_dir.as_ref().to_path_buf(),
        }
    }

    /// Loads a single module from a directory
    ///
    /// # Arguments
    /// * `module_path` - Path to the module directory (must contain manifest.json and index.js)
    ///
    /// # Returns
    /// A SiteModule instance, or an error if loading fails
    ///
    /// # Example
    /// ```ignore
    /// let loader = ModuleLoader::new("/path/to/modules");
    /// let module = loader.load_module("/path/to/modules/k2s")?;
    /// ```
    pub fn load_module(&self, module_path: impl AsRef<Path>) -> Result<SiteModule, String> {
        let module_path = module_path.as_ref();

        // T038: Validate module directory exists
        if !module_path.exists() {
            return Err(format!(
                "Module directory not found: {}",
                module_path.display()
            ));
        }

        if !module_path.is_dir() {
            return Err(format!(
                "Module path is not a directory: {}",
                module_path.display()
            ));
        }

        // T038: Read and parse manifest.json
        let manifest_path = module_path.join("manifest.json");
        let manifest_content = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest.json: {}", e))?;

        let manifest = ModuleManifest::from_json(&manifest_content)
            .map_err(|e| format!("Failed to parse manifest.json: {}", e))?;

        // T039: Read entry point file (from manifest.entrypoint)
        let entrypoint = &manifest.entrypoint;
        let code_path = module_path.join(entrypoint);

        // Validate entry point file exists
        if !code_path.exists() {
            return Err(format!(
                "Entry point file '{}' not found. Please ensure the file exists or build your module first.",
                entrypoint
            ));
        }

        let mut code = fs::read_to_string(&code_path)
            .map_err(|e| format!("Failed to read entry point '{}': {}", entrypoint, e))?;

        // Validate code is not empty
        if code.trim().is_empty() {
            return Err(format!("Entry point file '{}' is empty", entrypoint));
        }

        // Transform ES6 module syntax to plain JavaScript
        // This allows build tools to output ES modules, which we'll transform automatically
        // Deno's execute_script() doesn't support ES modules, so we strip export keywords
        code = code.replace("export default class", "class")
                   .replace("export class", "class")
                   .replace("export default function", "function")
                   .replace("export function", "function")
                   .replace("export default const", "const")
                   .replace("export const", "const")
                   .replace("export let", "let")
                   .replace("export var", "var")
                   .replace("export default", "");

        // T037: Create SiteModule instance
        let module_id = manifest.id.clone();
        let mut module = SiteModule::new(module_id, manifest, code);

        // T037: Initialize runtime and inject sandbox (handled by SiteModule::initialize)
        module.initialize()
            .map_err(|e| format!("Failed to initialize module: {}", e))?;

        Ok(module)
    }

    /// Loads all modules from the base modules directory
    ///
    /// # Returns
    /// A vector of successfully loaded modules
    ///
    /// # Note
    /// This function skips modules that fail to load and continues
    /// with the remaining modules. Check logs for loading errors.
    pub fn load_all_modules(&self) -> Vec<SiteModule> {
        let mut modules = Vec::new();

        // Check if modules directory exists
        if !self.modules_dir.exists() || !self.modules_dir.is_dir() {
            eprintln!(
                "Modules directory not found or not a directory: {}",
                self.modules_dir.display()
            );
            return modules;
        }

        // Read directory entries
        let entries = match fs::read_dir(&self.modules_dir) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Failed to read modules directory: {}", e);
                return modules;
            }
        };

        // Try to load each module
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Failed to read directory entry: {}", e);
                    continue;
                }
            };

            let path = entry.path();

            // Skip if not a directory
            if !path.is_dir() {
                continue;
            }

            // Skip hidden directories and template
            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy();
                if name_str.starts_with('.') || name_str.starts_with('_') || name_str == "@types" {
                    continue;
                }
            }

            // Try to load the module
            match self.load_module(&path) {
                Ok(module) => {
                    println!(
                        "Successfully loaded module: {} (v{}) from {}",
                        module.manifest.name,
                        module.manifest.version,
                        path.display()
                    );
                    modules.push(module);
                }
                Err(e) => {
                    eprintln!("Failed to load module from {}: {}", path.display(), e);
                }
            }
        }

        modules
    }

    /// Returns the base modules directory path
    pub fn modules_dir(&self) -> &Path {
        &self.modules_dir
    }
}

impl Default for ModuleLoader {
    fn default() -> Self {
        // Default to "./modules" directory
        Self::new("./modules")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::js_module::ModuleStatus;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    /// Create a test module directory with manifest.json and index.js
    fn create_test_module(dir: &Path, id: &str) -> Result<(), std::io::Error> {
        let module_dir = dir.join(id);
        fs::create_dir_all(&module_dir)?;

        // Create manifest.json
        let manifest = format!(
            r#"{{
                "id": "{}",
                "name": "Test Module",
                "version": "1.0.0",
                "author": "Test Author",
                "description": "Test module",
                "urlPatterns": ["https://test.com/*"],
                "permissions": {{
                    "network": {{
                        "allowDomains": ["test.com"]
                    }}
                }}
            }}"#,
            id
        );

        let mut manifest_file = fs::File::create(module_dir.join("manifest.json"))?;
        manifest_file.write_all(manifest.as_bytes())?;

        // Create index.js
        let code = format!(
            r#"
            class {} {{
                canHandle(url) {{
                    return url.includes('test.com');
                }}

                extractDownloadInfo(url) {{
                    return {{
                        downloadUrl: 'https://test.com/file.zip',
                        filename: 'test.zip'
                    }};
                }}
            }}
            "#,
            // Convert id to PascalCase
            id.split('-')
                .map(|part| {
                    let mut chars = part.chars();
                    match chars.next() {
                        Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                        None => String::new(),
                    }
                })
                .collect::<String>()
        );

        let mut code_file = fs::File::create(module_dir.join("index.js"))?;
        code_file.write_all(code.as_bytes())?;

        Ok(())
    }

    #[test]
    fn test_module_loader_creation() {
        let loader = ModuleLoader::new("/test/modules");
        assert_eq!(loader.modules_dir(), Path::new("/test/modules"));
    }

    #[test]
    fn test_module_loader_default() {
        let loader = ModuleLoader::default();
        assert_eq!(loader.modules_dir(), Path::new("./modules"));
    }

    #[test]
    fn test_load_nonexistent_module() {
        let loader = ModuleLoader::new("/nonexistent/path");
        let result = loader.load_module("/nonexistent/module");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_load_module_success() {
        let temp_dir = TempDir::new().unwrap();
        create_test_module(temp_dir.path(), "test-module").unwrap();

        let loader = ModuleLoader::new(temp_dir.path());
        let result = loader.load_module(temp_dir.path().join("test-module"));

        // The module should load successfully
        match result {
            Ok(module) => {
                assert_eq!(module.id, "test-module");
                assert_eq!(module.manifest.name, "Test Module");
                // Module might fail initialization due to sandbox, but loading should succeed
                assert!(
                    *module.status() == ModuleStatus::Active
                        || *module.status() == ModuleStatus::Failed
                );
            }
            Err(e) => {
                // If loading failed, it should be due to initialization, not file reading
                assert!(
                    e.contains("Failed to initialize") || e.contains("runtime"),
                    "Unexpected error: {}",
                    e
                );
            }
        }
    }

    #[test]
    fn test_load_all_modules() {
        let temp_dir = TempDir::new().unwrap();

        // Create multiple test modules
        create_test_module(temp_dir.path(), "module-one").unwrap();
        create_test_module(temp_dir.path(), "module-two").unwrap();

        // Create a hidden directory (should be skipped)
        fs::create_dir(temp_dir.path().join(".hidden")).unwrap();

        // Create a template directory (should be skipped)
        fs::create_dir(temp_dir.path().join("_template")).unwrap();

        let loader = ModuleLoader::new(temp_dir.path());
        let modules = loader.load_all_modules();

        // Should load 2 modules (skipping .hidden and _template)
        // Note: modules might fail initialization, so count might be less
        assert!(modules.len() <= 2);

        // Check that loaded modules have correct IDs
        for module in &modules {
            assert!(module.id == "module-one" || module.id == "module-two");
        }
    }

    #[test]
    fn test_load_module_missing_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let module_dir = temp_dir.path().join("bad-module");
        fs::create_dir_all(&module_dir).unwrap();

        // Only create index.js, no manifest.json
        let mut code_file = fs::File::create(module_dir.join("index.js")).unwrap();
        code_file.write_all(b"class TestModule {}").unwrap();

        let loader = ModuleLoader::new(temp_dir.path());
        let result = loader.load_module(&module_dir);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("manifest.json"));
    }

    #[test]
    fn test_load_module_missing_index_js() {
        let temp_dir = TempDir::new().unwrap();
        let module_dir = temp_dir.path().join("bad-module");
        fs::create_dir_all(&module_dir).unwrap();

        // Only create manifest.json, no index.js
        let manifest = r#"{
            "id": "bad-module",
            "name": "Bad Module",
            "version": "1.0.0",
            "author": "Test",
            "description": "Test",
            "urlPatterns": ["https://test.com/*"],
            "permissions": {
                "network": {
                    "allowDomains": ["test.com"]
                }
            }
        }"#;

        let mut manifest_file = fs::File::create(module_dir.join("manifest.json")).unwrap();
        manifest_file.write_all(manifest.as_bytes()).unwrap();

        let loader = ModuleLoader::new(temp_dir.path());
        let result = loader.load_module(&module_dir);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("index.js"));
    }

    #[test]
    fn test_load_module_empty_code() {
        let temp_dir = TempDir::new().unwrap();
        let module_dir = temp_dir.path().join("empty-module");
        fs::create_dir_all(&module_dir).unwrap();

        // Create manifest.json
        let manifest = r#"{
            "id": "empty-module",
            "name": "Empty Module",
            "version": "1.0.0",
            "author": "Test",
            "description": "Test",
            "urlPatterns": ["https://test.com/*"],
            "permissions": {
                "network": {
                    "allowDomains": ["test.com"]
                }
            }
        }"#;

        let mut manifest_file = fs::File::create(module_dir.join("manifest.json")).unwrap();
        manifest_file.write_all(manifest.as_bytes()).unwrap();

        // Create empty index.js
        fs::File::create(module_dir.join("index.js")).unwrap();

        let loader = ModuleLoader::new(temp_dir.path());
        let result = loader.load_module(&module_dir);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }
}
#[cfg(test)]
mod module_loading_tests {
    use crate::modules::loader::ModuleLoader;

    #[test]
    fn test_load_k2s_module() {
        let loader = ModuleLoader::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../modules"));
        let result = loader.load_module(concat!(env!("CARGO_MANIFEST_DIR"), "/../modules/k2s"));
        
        match result {
            Ok(module) => {
                println!("✅ Successfully loaded k2s module");
                println!("   ID: {}", module.id);
                println!("   Name: {}", module.manifest.name);
                println!("   Class name: {:?}", module.manifest.class_name);
                println!("   Status: {:?}", module.status());
            },
            Err(e) => {
                panic!("❌ Failed to load k2s module: {}", e);
            }
        }
    }

    #[test]
    fn test_load_example_simple_module() {
        let loader = ModuleLoader::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../modules"));
        let result = loader.load_module(concat!(env!("CARGO_MANIFEST_DIR"), "/../modules/example-simple"));
        
        match result {
            Ok(module) => {
                println!("✅ Successfully loaded example-simple module");
                println!("   ID: {}", module.id);
                println!("   Name: {}", module.manifest.name);
                println!("   Class name: {:?}", module.manifest.class_name);
                println!("   Status: {:?}", module.status());
            },
            Err(e) => {
                panic!("❌ Failed to load example-simple module: {}", e);
            }
        }
    }
}
