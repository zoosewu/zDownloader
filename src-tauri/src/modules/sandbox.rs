// Module Sandbox Environment (Feature 004)
//
// This module implements the security sandbox for JavaScript modules,
// restricting access to dangerous APIs and protecting against sandbox escape.

/// Module sandbox manager
pub struct ModuleSandbox {
    /// Whether to enable strict mode
    strict_mode: bool,
}

impl ModuleSandbox {
    /// Create a new sandbox with security restrictions
    pub fn new() -> Self {
        Self {
            strict_mode: true,
        }
    }

    /// Get sandbox initialization script
    /// This script disables dangerous APIs and freezes prototypes
    pub fn get_init_script() -> &'static str {
        r#"
        // Sandbox Initialization Script
        // This script runs before any module code to establish security restrictions

        (function() {
            'use strict';

            // 1. First, protect __proto__ and constructor (before freezing)
            try {
                Object.defineProperty(Object.prototype, '__proto__', {
                    configurable: false,
                    enumerable: false,
                    get: function() {
                        return Object.getPrototypeOf(this);
                    },
                    set: function() {
                        throw new Error('Sandbox violation: Cannot modify __proto__');
                    }
                });
            } catch (e) {
                // __proto__ might already be protected
            }

            try {
                Object.defineProperty(Object.prototype, 'constructor', {
                    configurable: false,
                    enumerable: false,
                    writable: false
                });
            } catch (e) {
                // constructor might already be protected
            }

            // 2. Freeze built-in prototypes
            Object.freeze(Object.prototype);
            Object.freeze(Array.prototype);
            Object.freeze(String.prototype);
            Object.freeze(Number.prototype);
            Object.freeze(Boolean.prototype);
            Object.freeze(Function.prototype);
            Object.freeze(Date.prototype);
            Object.freeze(RegExp.prototype);
            Object.freeze(Error.prototype);
            Object.freeze(Map.prototype);
            Object.freeze(Set.prototype);
            Object.freeze(WeakMap.prototype);
            Object.freeze(WeakSet.prototype);
            Object.freeze(Promise.prototype);

            // 3. Delete dangerous global functions
            delete globalThis.eval;
            delete globalThis.Function;
            delete globalThis.GeneratorFunction;
            delete globalThis.AsyncFunction;
            delete globalThis.AsyncGeneratorFunction;

            // 4. Delete dynamic code execution methods
            if (globalThis.setTimeout) delete globalThis.setTimeout;
            if (globalThis.setInterval) delete globalThis.setInterval;
            if (globalThis.setImmediate) delete globalThis.setImmediate;

            // 5. Freeze globalThis (last step)
            Object.freeze(globalThis);

            // Sandbox initialized successfully
        })();
        "#
    }

    /// Inject sandbox restrictions into JavaScript runtime
    pub fn inject_restrictions(&self, runtime: &mut crate::modules::js_runtime::JsRuntimeHandle) -> Result<(), String> {
        // Execute the sandbox initialization script
        runtime.execute_script("__sandbox_init__", Self::get_init_script())?;
        Ok(())
    }

    /// Detect sandbox violations from error messages
    pub fn detect_violation(&self, error: &str) -> bool {
        // Check for common sandbox violation patterns
        let violation_patterns = [
            "Sandbox violation",
            "eval is not defined",
            "Function is not defined",
            "Cannot modify __proto__",
            "Cannot modify frozen object",
            "Cannot add property",
            "Cannot delete property",
        ];

        violation_patterns.iter().any(|pattern| error.contains(pattern))
    }

    /// Validate that sandbox is properly initialized
    pub fn validate_sandbox(&self, runtime: &mut crate::modules::js_runtime::JsRuntimeHandle) -> Result<(), String> {
        // Test that dangerous APIs are disabled
        let test_script = r#"
        (function() {
            const violations = [];

            // Check that eval is deleted
            if (typeof eval !== 'undefined') {
                violations.push('eval is still accessible');
            }

            // Check that Function constructor is deleted
            if (typeof Function !== 'undefined') {
                violations.push('Function constructor is still accessible');
            }

            // Check that Object.prototype is frozen
            try {
                Object.prototype.testProperty = 'test';
                violations.push('Object.prototype is not frozen');
            } catch (e) {
                // Expected - prototype is frozen
            }

            if (violations.length > 0) {
                throw new Error('Sandbox validation failed: ' + violations.join(', '));
            }

            return 'Sandbox validated successfully';
        })()
        "#;

        runtime.execute_script("__sandbox_validate__", test_script)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::js_runtime::JsRuntimeHandle;

    #[test]
    fn test_sandbox_creation() {
        let sandbox = ModuleSandbox::new();
        assert!(sandbox.strict_mode);
    }

    #[test]
    fn test_sandbox_init_script_not_empty() {
        let script = ModuleSandbox::get_init_script();
        assert!(!script.is_empty());
        assert!(script.contains("delete globalThis.eval"));
        assert!(script.contains("Object.freeze"));
    }

    #[test]
    fn test_detect_violation() {
        let sandbox = ModuleSandbox::new();

        assert!(sandbox.detect_violation("Sandbox violation: Cannot modify"));
        assert!(sandbox.detect_violation("eval is not defined"));
        assert!(sandbox.detect_violation("Cannot modify __proto__"));
        assert!(!sandbox.detect_violation("Normal error message"));
    }

    #[test]
    fn test_sandbox_injection() {
        let mut runtime = JsRuntimeHandle::new().unwrap();
        let sandbox = ModuleSandbox::new();

        let result = sandbox.inject_restrictions(&mut runtime);
        assert!(result.is_ok(), "Should inject sandbox restrictions successfully: {:?}", result.err());
    }
}
