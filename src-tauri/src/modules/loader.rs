// Dynamic module loader using libloading
use super::interface::{BoxedModule, ModuleError, ModuleResult};
use libloading::{Library, Symbol};
use std::path::Path;

/// Type signature for the module creation function
/// All modules must export a function with this signature named "create_module"
type CreateModuleFn = unsafe fn() -> BoxedModule;

/// Module loader that can load website modules from dynamic libraries
pub struct ModuleLoader {
    /// Loaded libraries (kept alive to prevent unloading)
    _libraries: Vec<Library>,
}

impl ModuleLoader {
    /// Creates a new module loader
    pub fn new() -> Self {
        Self {
            _libraries: Vec::new(),
        }
    }

    /// Loads a module from a dynamic library file
    ///
    /// # Arguments
    /// * `path` - Path to the dynamic library (.so/.dll/.dylib)
    ///
    /// # Returns
    /// A boxed module instance, or an error if loading fails
    ///
    /// # Safety
    /// This function loads and executes code from a dynamic library.
    /// The caller must ensure the library is trustworthy.
    ///
    /// # Example
    /// ```ignore
    /// let mut loader = ModuleLoader::new();
    /// let module = loader.load_module("/path/to/module.so")?;
    /// ```
    pub fn load_module(&mut self, path: impl AsRef<Path>) -> ModuleResult<BoxedModule> {
        let path = path.as_ref();

        // Verify file exists
        if !path.exists() {
            return Err(ModuleError::InternalError(format!(
                "Module file not found: {}",
                path.display()
            )));
        }

        // Load the library
        let lib = unsafe {
            Library::new(path).map_err(|e| {
                ModuleError::InternalError(format!("Failed to load library: {}", e))
            })?
        };

        // Look for the create_module function
        let module = unsafe {
            let create_fn: Symbol<CreateModuleFn> = lib
                .get(b"create_module")
                .map_err(|e| {
                    ModuleError::InternalError(format!(
                        "Failed to find create_module function: {}",
                        e
                    ))
                })?;

            // Call the creation function
            create_fn()
        };

        // Keep the library alive
        self._libraries.push(lib);

        Ok(module)
    }

    /// Loads all modules from a directory
    ///
    /// # Arguments
    /// * `dir` - Directory containing module files
    ///
    /// # Returns
    /// A vector of successfully loaded modules
    ///
    /// # Note
    /// This function skips files that fail to load and continues
    /// with the remaining files. Check logs for loading errors.
    pub fn load_modules_from_dir(&mut self, dir: impl AsRef<Path>) -> Vec<BoxedModule> {
        let dir = dir.as_ref();
        let mut modules = Vec::new();

        // Check if directory exists
        if !dir.exists() || !dir.is_dir() {
            eprintln!("Module directory not found or not a directory: {}", dir.display());
            return modules;
        }

        // Determine the library extension based on platform
        let lib_extension = if cfg!(target_os = "windows") {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        };

        // Read directory entries
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Failed to read module directory: {}", e);
                return modules;
            }
        };

        // Try to load each library file
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Failed to read directory entry: {}", e);
                    continue;
                }
            };

            let path = entry.path();

            // Skip if not a file or wrong extension
            if !path.is_file() {
                continue;
            }

            if let Some(ext) = path.extension() {
                if ext != lib_extension {
                    continue;
                }
            } else {
                continue;
            }

            // Try to load the module
            match self.load_module(&path) {
                Ok(module) => {
                    println!("Successfully loaded module from: {}", path.display());
                    modules.push(module);
                }
                Err(e) => {
                    eprintln!("Failed to load module from {}: {}", path.display(), e);
                }
            }
        }

        modules
    }

    /// Returns the number of loaded libraries
    pub fn loaded_count(&self) -> usize {
        self._libraries.len()
    }
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

// Note: We cannot safely unload modules because Rust's trait objects
// may have references into the library code. The libraries will be
// unloaded when the ModuleLoader is dropped, but only after all
// modules have been dropped.

impl Drop for ModuleLoader {
    fn drop(&mut self) {
        // Libraries will be automatically unloaded when dropped
        // This is safe only if all modules have been dropped first
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_loader_creation() {
        let loader = ModuleLoader::new();
        assert_eq!(loader.loaded_count(), 0);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let mut loader = ModuleLoader::new();
        let result = loader.load_module("/nonexistent/path/to/module.so");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_modules_from_nonexistent_dir() {
        let mut loader = ModuleLoader::new();
        let modules = loader.load_modules_from_dir("/nonexistent/directory");
        assert_eq!(modules.len(), 0);
    }

    #[test]
    fn test_library_extension() {
        let ext = if cfg!(target_os = "windows") {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        };

        assert!(!ext.is_empty());
    }
}
