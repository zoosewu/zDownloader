// Serialization Consistency Tests
//
// These tests ensure that Rust data structures serialize to JSON in a format
// that matches the TypeScript interface definitions in the frontend.

use super::super::manifest::{ModuleManifest, ModulePermissions, NetworkPermission, CredentialPermission};
use super::super::js_module::ModuleStatus;
use serde_json;

#[test]
fn test_module_status_serialization_uses_camel_case() {
    // Test that ModuleStatus serializes to lowercase (camelCase)
    // Frontend expects: 'pending', 'loading', 'active', 'failed', 'suspended', 'unloaded'

    let statuses = vec![
        (ModuleStatus::Pending, "pending"),
        (ModuleStatus::Loading, "loading"),
        (ModuleStatus::Active, "active"),
        (ModuleStatus::Failed, "failed"),
        (ModuleStatus::Suspended, "suspended"),
        (ModuleStatus::Unloaded, "unloaded"),
    ];

    for (status, expected) in statuses {
        let json = serde_json::to_string(&status).unwrap();
        let json = json.trim_matches('"'); // Remove quotes from JSON string
        assert_eq!(
            json, expected,
            "ModuleStatus::{:?} should serialize to \"{}\"",
            status, expected
        );
    }
}

#[test]
fn test_module_status_deserialization_from_camel_case() {
    // Test that ModuleStatus can be deserialized from camelCase strings
    let statuses = vec![
        ("\"pending\"", ModuleStatus::Pending),
        ("\"loading\"", ModuleStatus::Loading),
        ("\"active\"", ModuleStatus::Active),
        ("\"failed\"", ModuleStatus::Failed),
        ("\"suspended\"", ModuleStatus::Suspended),
        ("\"unloaded\"", ModuleStatus::Unloaded),
    ];

    for (json, expected) in statuses {
        let result: Result<ModuleStatus, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "Should deserialize \"{}\"", json);
        assert_eq!(result.unwrap(), expected);
    }
}

#[test]
fn test_manifest_serialization_uses_camel_case() {
    // Test that ModuleManifest fields serialize to camelCase
    let manifest = ModuleManifest {
        id: "test".to_string(),
        name: "Test Module".to_string(),
        version: "1.0.0".to_string(),
        author: "Test".to_string(),
        description: "Test".to_string(),
        homepage: None,
        url_patterns: vec!["https://example.com/*".to_string()],
        permissions: ModulePermissions {
            network: NetworkPermission {
                allow_domains: vec!["example.com".to_string()],
            },
            credentials: Some(CredentialPermission {
                required: true,
                credential_keys: vec!["username".to_string()],
            }),
        },
        entrypoint: "index.js".to_string(),
        class_name: None,
    };

    let json = serde_json::to_string(&manifest).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Verify camelCase field names exist
    assert!(parsed.get("urlPatterns").is_some(), "Should have 'urlPatterns' (camelCase)");
    assert!(parsed.get("url_patterns").is_none(), "Should NOT have 'url_patterns' (snake_case)");

    // Verify nested camelCase
    let permissions = parsed.get("permissions").unwrap();
    let network = permissions.get("network").unwrap();
    assert!(network.get("allowDomains").is_some(), "Should have 'allowDomains' (camelCase)");
    assert!(network.get("allow_domains").is_none(), "Should NOT have 'allow_domains' (snake_case)");

    let credentials = permissions.get("credentials").unwrap();
    assert!(credentials.get("credentialKeys").is_some(), "Should have 'credentialKeys' (camelCase)");
    assert!(credentials.get("credential_keys").is_none(), "Should NOT have 'credential_keys' (snake_case)");
}

#[test]
fn test_manifest_roundtrip_serialization() {
    // Test that manifest can be serialized and deserialized without data loss
    let original = ModuleManifest {
        id: "k2s".to_string(),
        name: "Keep2Share Downloader".to_string(),
        version: "1.0.0".to_string(),
        author: "Zoo Downloader Community".to_string(),
        description: "Test description".to_string(),
        homepage: Some("https://github.com/zoo-downloader/modules/k2s".to_string()),
        url_patterns: vec![
            "https://k2s.cc/file/*".to_string(),
            "https://*.k2s.cc/file/*".to_string(),
        ],
        permissions: ModulePermissions {
            network: NetworkPermission {
                allow_domains: vec![
                    "k2s.cc".to_string(),
                    "*.k2s.cc".to_string(),
                ],
            },
            credentials: Some(CredentialPermission {
                required: true,
                credential_keys: vec![
                    "username".to_string(),
                    "password".to_string(),
                    "apiKey".to_string(),
                ],
            }),
        },
        entrypoint: "index.js".to_string(),
        class_name: None,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&original).unwrap();

    // Deserialize back
    let deserialized: ModuleManifest = serde_json::from_str(&json).unwrap();

    // Verify all fields match
    assert_eq!(deserialized.id, original.id);
    assert_eq!(deserialized.name, original.name);
    assert_eq!(deserialized.version, original.version);
    assert_eq!(deserialized.author, original.author);
    assert_eq!(deserialized.description, original.description);
    assert_eq!(deserialized.homepage, original.homepage);
    assert_eq!(deserialized.url_patterns, original.url_patterns);
    assert_eq!(
        deserialized.permissions.network.allow_domains,
        original.permissions.network.allow_domains
    );
    assert_eq!(
        deserialized.permissions.credentials.as_ref().unwrap().required,
        original.permissions.credentials.as_ref().unwrap().required
    );
    assert_eq!(
        deserialized.permissions.credentials.as_ref().unwrap().credential_keys,
        original.permissions.credentials.as_ref().unwrap().credential_keys
    );
}
