// Phase 1 Setup Tests
//
// Tests for basic setup and infrastructure validation

use super::super::manifest::{ModuleManifest, ModulePermissions, NetworkPermission};
use std::path::Path;

#[test]
fn test_module_directories_exist() {
    // Test that required module directories are created
    assert!(Path::new("../modules").exists(), "modules/ directory should exist");
    assert!(Path::new("../modules/_template").exists(), "modules/_template/ directory should exist");
    assert!(Path::new("../modules/k2s").exists(), "modules/k2s/ directory should exist");
}

#[test]
fn test_audit_log_directory_exists() {
    // Test that audit log directory is created
    assert!(Path::new("../logs/audit").exists(), "logs/audit/ directory should exist");
}

#[test]
fn test_k2s_manifest_exists() {
    // Test that K2S manifest file exists
    let manifest_path = Path::new("../modules/k2s/manifest.json");
    assert!(manifest_path.exists(), "K2S manifest.json should exist");
}

#[test]
fn test_k2s_manifest_parse() {
    // Test that K2S manifest can be parsed
    let manifest_json = r#"{
        "id": "k2s",
        "name": "Keep2Share Downloader",
        "version": "1.0.0",
        "author": "Zoo Downloader Community",
        "description": "Test description",
        "urlPatterns": [
            "https://k2s.cc/file/*"
        ],
        "permissions": {
            "network": {
                "allowDomains": ["k2s.cc"]
            },
            "credentials": {
                "required": true,
                "credentialKeys": ["username", "password"]
            }
        }
    }"#;

    let result = ModuleManifest::from_json(manifest_json);
    assert!(result.is_ok(), "Should parse valid manifest JSON");

    let manifest = result.unwrap();
    assert_eq!(manifest.id, "k2s");
    assert_eq!(manifest.name, "Keep2Share Downloader");
    assert_eq!(manifest.version, "1.0.0");
    assert_eq!(manifest.url_patterns.len(), 1);
    assert_eq!(manifest.permissions.network.allow_domains.len(), 1);
    assert!(manifest.permissions.credentials.is_some());
}

#[test]
fn test_manifest_validation_basic() {
    // Test basic manifest validation
    let valid_manifest = ModuleManifest {
        id: "test".to_string(),
        name: "Test Module".to_string(),
        version: "1.0.0".to_string(),
        author: "Test Author".to_string(),
        description: "Test description".to_string(),
        homepage: None,
        url_patterns: vec!["https://example.com/*".to_string()],
        permissions: ModulePermissions {
            network: NetworkPermission {
                allow_domains: vec!["example.com".to_string()],
            },
            credentials: None,
        },
        entrypoint: "index.js".to_string(),
        class_name: None,
    };

    assert!(valid_manifest.validate().is_ok(), "Valid manifest should pass validation");

    // Test empty id
    let invalid_manifest = ModuleManifest {
        id: "".to_string(),
        ..valid_manifest.clone()
    };
    assert!(invalid_manifest.validate().is_err(), "Empty id should fail validation");

    // Test empty name
    let invalid_manifest = ModuleManifest {
        name: "".to_string(),
        ..valid_manifest.clone()
    };
    assert!(invalid_manifest.validate().is_err(), "Empty name should fail validation");

    // Test empty version
    let invalid_manifest = ModuleManifest {
        version: "".to_string(),
        ..valid_manifest
    };
    assert!(invalid_manifest.validate().is_err(), "Empty version should fail validation");
}

#[test]
fn test_manifest_invalid_json() {
    // Test that invalid JSON is rejected
    let invalid_json = r#"{ "id": "test", "invalid": }"#;
    let result = ModuleManifest::from_json(invalid_json);
    assert!(result.is_err(), "Invalid JSON should fail to parse");
}

#[test]
fn test_manifest_missing_required_fields() {
    // Test that missing required fields are detected
    let missing_id = r#"{
        "name": "Test",
        "version": "1.0.0",
        "author": "Test",
        "description": "Test",
        "urlPatterns": [],
        "permissions": {
            "network": {
                "allowDomains": []
            }
        }
    }"#;

    let result = ModuleManifest::from_json(missing_id);
    assert!(result.is_err(), "Missing 'id' field should fail to parse");
}
