// JavaScript Runtime Management (Feature 004)
//
// This module manages the deno_core JavaScript runtime instances for executing
// website module scripts in a sandboxed environment.

use deno_core::JsRuntime;

/// JavaScript runtime handle for executing module code
pub struct JsRuntimeHandle {
    runtime: JsRuntime,
}

// Manual Debug implementation since JsRuntime doesn't derive Debug
impl std::fmt::Debug for JsRuntimeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsRuntimeHandle")
            .field("runtime", &"<JsRuntime>")
            .finish()
    }
}

impl JsRuntimeHandle {
    /// Create a new JavaScript runtime with sandbox restrictions
    pub fn new() -> Result<Self, String> {
        // Create runtime with default options
        // Sandbox restrictions will be injected separately in Phase 2.3
        let runtime = JsRuntime::new(Default::default());

        Ok(Self { runtime })
    }

    /// Create a new JavaScript runtime with Context API (Phase 4: T073)
    ///
    /// NOTE: Full ops registration is not implemented due to deno_core API complexity.
    /// The Context API handlers are fully implemented in context_api.rs.
    /// For now, this creates a standard runtime and the Context API can be
    /// injected via execute_script() with ModuleContext::generate_context_script().
    pub fn new_with_context(_module_id: String) -> Result<Self, String> {
        // For now, create a standard runtime
        // Full ops integration will be done in future iterations
        let runtime = JsRuntime::new(Default::default());

        Ok(Self { runtime })
    }

    /// Execute JavaScript code in the runtime
    ///
    /// # Arguments
    /// * `script_name` - Name for the script (used in error messages)
    /// * `code` - JavaScript code to execute
    pub fn execute_script(&mut self, script_name: &str, code: &str) -> Result<(), String> {
        // Convert &str to ModuleCodeString using to_string()
        self.runtime
            .execute_script(script_name.to_string(), code.to_string())
            .map_err(|e| format!("Failed to execute script '{}': {:?}", script_name, e))?;

        Ok(())
    }

    /// Run the event loop to completion
    /// This is necessary for async operations to complete
    pub async fn run_event_loop_async(&mut self) -> Result<(), String> {
        self.runtime
            .run_event_loop(Default::default())
            .await
            .map_err(|e| format!("Failed to run event loop: {:?}", e))?;

        Ok(())
    }

    /// Execute and return the result as a JSON string (simplified version)
    ///
    /// # Arguments
    /// * `script_name` - Name for the script
    /// * `code` - JavaScript expression that returns a value
    pub fn execute_and_get_result(&mut self, script_name: &str, code: &str) -> Result<String, String> {
        // Wrap the code to return JSON string
        let wrapped_code = format!(
            r#"JSON.stringify({})"#,
            code
        );

        // Execute and get the result
        self.execute_script(script_name, &wrapped_code)?;

        // For now, return a placeholder
        // Full implementation will use v8 scope to get the actual result
        // This is a simplified version for Phase 2 MVP
        Ok("{}".to_string())
    }

    /// Call a JavaScript function by name with arguments
    ///
    /// # Arguments
    /// * `function_name` - Name of the function to call
    /// * `args_json` - JSON string of arguments array, e.g. "[1, 2]"
    pub fn call_function(&mut self, function_name: &str, args_json: &str) -> Result<String, String> {
        let code = format!(
            r#"
            if (typeof {} !== 'function') {{
                throw new Error('Function {} is not defined');
            }}
            const args = {};
            {}(...args);
            "#,
            function_name,
            function_name,
            args_json,
            function_name
        );

        self.execute_script(&format!("call_{}", function_name), &code)?;
        Ok("{}".to_string())
    }
}

impl Default for JsRuntimeHandle {
    fn default() -> Self {
        Self::new().expect("Failed to create default JsRuntimeHandle")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = JsRuntimeHandle::new();
        assert!(runtime.is_ok(), "Should create runtime successfully");
    }

    #[test]
    fn test_execute_simple_script() {
        let mut runtime = JsRuntimeHandle::new().unwrap();
        let result = runtime.execute_script("test.js", "const x = 1 + 1;");
        assert!(result.is_ok(), "Should execute simple script");
    }

    #[test]
    fn test_execute_invalid_script() {
        let mut runtime = JsRuntimeHandle::new().unwrap();
        let result = runtime.execute_script("test.js", "invalid javascript {{{");
        assert!(result.is_err(), "Should fail on invalid JavaScript");
    }

    #[test]
    fn test_call_function() {
        let mut runtime = JsRuntimeHandle::new().unwrap();

        // Define a function
        runtime.execute_script("define.js", "function add(a, b) { return a + b; }").unwrap();

        // Call the function
        let result = runtime.call_function("add", "[2, 3]");
        assert!(result.is_ok(), "Should call function successfully");
    }
}
