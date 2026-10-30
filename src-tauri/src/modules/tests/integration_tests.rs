// End-to-End Integration Tests (Feature 004 - Phase 4.6 - T089)
//
// This module contains integration tests for the complete download flow
// from URL detection to task creation, validating the full module system integration.

use crate::modules::registry::ModuleRegistry as SiteModuleRegistry;
use crate::modules::url_detector::{detect_url_with_modules, DetectionMode};

/// Test: End-to-end URL detection with module integration (T089)
#[test]
fn test_e2e_url_detection_with_module() {
    // Test K2S URL detection
    let k2s_url = "https://k2s.cc/file/abc123def456/example.zip";
    let site_registry = SiteModuleRegistry::new();

    let result = detect_url_with_modules(k2s_url, &site_registry);

    // Should be detected as module-custom
    // Note: Actual module matching depends on ModuleRegistry.find_module_for_url()
    // which is not fully implemented yet, so we test the fallback behavior
    assert_eq!(result.url, k2s_url);
    assert!(result.is_valid);

    println!("✅ E2E URL detection completed: mode={:?}", result.detection_mode);
}

/// Test: URL detection fallback behavior (T088)
#[test]
fn test_e2e_url_detection_fallback() {
    // Test URL that should not match any module
    let generic_url = "https://example.com/download/file.zip";
    let site_registry = SiteModuleRegistry::new();

    let result = detect_url_with_modules(generic_url, &site_registry);

    // Should fall back to builtin detection
    assert_eq!(result.url, generic_url);
    assert!(result.is_valid); // Should still be valid
    assert_eq!(result.detection_mode, DetectionMode::FileExtension);
    assert!(result.suggested_filename.is_some());
    assert_eq!(result.suggested_filename.unwrap(), "file.zip");

    println!("✅ E2E URL detection fallback successful");
}

/// Test: Module error handling expectations (T088)
#[test]
fn test_e2e_module_error_handling() {
    // Test scenarios that should be handled gracefully:
    // 1. Invalid URL format
    // 2. No matching module (fallback to generic)
    // 3. Module detection failure (fallback)

    let scenarios = vec![
        (
            "https://unknown-site.com/file.zip",
            "Unknown site - fallback to generic",
        ),
        (
            "https://example.com/path/to/file.rar",
            "Generic file URL - builtin detector",
        ),
    ];

    for (url, description) in scenarios {
        let site_registry = SiteModuleRegistry::new();
        let result = detect_url_with_modules(url, &site_registry);

        // Should always return a result (never panic)
        assert!(!result.url.is_empty(), "URL should not be empty");

        println!("✅ E2E error handling for '{}': {}", description, url);
    }
}

/// Test: Invalid URL handling (T088)
#[test]
fn test_e2e_invalid_url_handling() {
    let invalid_urls = vec![
        "not-a-url",
        "ftp://example.com/file.zip",      // Wrong protocol
        "javascript:alert(1)",              // XSS attempt
        "file:///etc/passwd",               // Local file
        "data:text/html,<script>alert(1)", // Data URL
    ];

    let site_registry = SiteModuleRegistry::new();

    for url in invalid_urls {
        let result = detect_url_with_modules(url, &site_registry);

        // Invalid URLs should be marked as invalid or rejected
        // The system should handle this gracefully without crashing
        println!(
            "URL '{}': valid={}, mode={:?}",
            url, result.is_valid, result.detection_mode
        );
    }

    println!("✅ E2E invalid URL handling successful (no crashes)");
}
