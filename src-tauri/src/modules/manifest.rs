// Module Manifest Parser and Validator (Feature 004)
//
// This module handles parsing and validation of module manifest.json files,
// including version checking, permission validation, and schema enforcement.

use serde::{Deserialize, Serialize};

/// Module manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub homepage: Option<String>,
    pub url_patterns: Vec<String>,
    pub permissions: ModulePermissions,

    /// Entry point file path (relative to module directory)
    /// Default: "index.js"
    /// Examples: "dist/index.js", "output/bundle.js", "build/main.js"
    #[serde(default = "default_entrypoint")]
    pub entrypoint: String,

    /// Optional: Explicit class name for module instantiation
    /// If not specified, class name will be auto-generated from module ID
    /// Example: "K2SModule", "SimpleExampleModule"
    #[serde(default)]
    pub class_name: Option<String>,
}

/// Default entrypoint for backward compatibility
fn default_entrypoint() -> String {
    "index.js".to_string()
}

/// Module permissions structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModulePermissions {
    pub network: NetworkPermission,
    pub credentials: Option<CredentialPermission>,
}

/// Network access permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkPermission {
    pub allow_domains: Vec<String>,
}

/// Credential access permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialPermission {
    pub required: bool,
    pub credential_keys: Vec<String>,
}

impl ModuleManifest {
    /// Parse manifest from JSON string
    pub fn from_json(json: &str) -> Result<Self, String> {
        let manifest: ModuleManifest = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse manifest JSON: {}", e))?;

        // Validate after parsing
        manifest.validate()?;

        Ok(manifest)
    }

    /// Validate manifest fields and constraints
    pub fn validate(&self) -> Result<(), String> {
        // 1. Validate required string fields are not empty
        if self.id.is_empty() {
            return Err("Manifest 'id' cannot be empty".to_string());
        }
        if self.name.is_empty() {
            return Err("Manifest 'name' cannot be empty".to_string());
        }
        if self.author.is_empty() {
            return Err("Manifest 'author' cannot be empty".to_string());
        }
        if self.description.is_empty() {
            return Err("Manifest 'description' cannot be empty".to_string());
        }

        // 2. Validate ID format (lowercase alphanumeric and hyphens only)
        if !self.id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err(format!(
                "Manifest 'id' must contain only lowercase letters, digits, and hyphens. Got: '{}'",
                self.id
            ));
        }

        // 3. Validate version using semver
        if let Err(e) = semver::Version::parse(&self.version) {
            return Err(format!(
                "Manifest 'version' is not a valid semantic version: {}. Error: {}",
                self.version, e
            ));
        }

        // 4. Validate URL patterns are not empty
        if self.url_patterns.is_empty() {
            return Err("Manifest 'urlPatterns' cannot be empty".to_string());
        }

        // 5. Validate each URL pattern
        for pattern in &self.url_patterns {
            if pattern.is_empty() {
                return Err("URL pattern cannot be empty".to_string());
            }
            // Basic URL pattern validation (must start with http:// or https://)
            if !pattern.starts_with("http://") && !pattern.starts_with("https://") {
                return Err(format!(
                    "URL pattern must start with 'http://' or 'https://'. Got: '{}'",
                    pattern
                ));
            }
        }

        // 6. Validate network permissions
        if self.permissions.network.allow_domains.is_empty() {
            return Err("Network 'allowDomains' cannot be empty".to_string());
        }

        // 7. Validate domain patterns
        for domain in &self.permissions.network.allow_domains {
            if domain.is_empty() {
                return Err("Domain in 'allowDomains' cannot be empty".to_string());
            }
            // Basic domain validation (allow wildcards)
            if !Self::is_valid_domain_pattern(domain) {
                return Err(format!(
                    "Invalid domain pattern: '{}'. Must be a valid domain or wildcard pattern.",
                    domain
                ));
            }
        }

        // 8. Validate credentials configuration if present
        if let Some(ref creds) = self.permissions.credentials {
            if creds.credential_keys.is_empty() {
                return Err("Credential 'credentialKeys' cannot be empty when credentials are required".to_string());
            }
            // Validate each credential key
            for key in &creds.credential_keys {
                if key.is_empty() {
                    return Err("Credential key cannot be empty".to_string());
                }
                if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    return Err(format!(
                        "Credential key must contain only alphanumeric characters and underscores. Got: '{}'",
                        key
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validate domain pattern format
    fn is_valid_domain_pattern(domain: &str) -> bool {
        // Allow wildcard patterns like *.example.com
        // Allow normal domains like example.com
        // Basic validation: must contain at least one dot (except wildcards)

        if domain == "*" {
            return false; // Pure wildcard not allowed
        }

        // Check for valid characters (alphanumeric, dots, hyphens, wildcards)
        if !domain.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '*') {
            return false;
        }

        // Count wildcards - only one allowed and must be at the start
        let wildcard_count = domain.matches('*').count();
        if wildcard_count > 1 {
            return false; // Multiple wildcards not allowed
        }

        // Wildcard can only appear at the start
        if domain.contains('*') && !domain.starts_with("*.") {
            return false;
        }

        // Must have at least one dot (for TLD)
        if !domain.contains('.') {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_manifest_parsing() {
        let json = r#"{
            "id": "test-module",
            "name": "Test Module",
            "version": "1.0.0",
            "author": "Test Author",
            "description": "A test module",
            "homepage": "https://example.com",
            "urlPatterns": ["https://example.com/*"],
            "permissions": {
                "network": {
                    "allowDomains": ["example.com", "*.example.com"]
                },
                "credentials": {
                    "required": true,
                    "credentialKeys": ["username", "password"]
                }
            }
        }"#;

        let manifest = ModuleManifest::from_json(json);
        assert!(manifest.is_ok());

        let manifest = manifest.unwrap();
        assert_eq!(manifest.id, "test-module");
        assert_eq!(manifest.name, "Test Module");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.author, "Test Author");
        assert_eq!(manifest.permissions.network.allow_domains.len(), 2);
    }

    #[test]
    fn test_manifest_without_credentials() {
        let json = r#"{
            "id": "simple-module",
            "name": "Simple Module",
            "version": "2.1.3",
            "author": "Author",
            "description": "Simple module without credentials",
            "urlPatterns": ["https://test.com/*"],
            "permissions": {
                "network": {
                    "allowDomains": ["test.com"]
                }
            }
        }"#;

        let manifest = ModuleManifest::from_json(json);
        assert!(manifest.is_ok());
        assert!(manifest.unwrap().permissions.credentials.is_none());
    }

    #[test]
    fn test_manifest_with_explicit_class_name() {
        let json = r#"{
            "id": "k2s",
            "name": "K2S Module",
            "version": "1.0.0",
            "author": "Author",
            "description": "Test module with explicit class name",
            "className": "K2SModule",
            "urlPatterns": ["https://k2s.cc/*"],
            "permissions": {
                "network": {
                    "allowDomains": ["k2s.cc"]
                }
            }
        }"#;

        let manifest = ModuleManifest::from_json(json);
        assert!(manifest.is_ok());
        let manifest = manifest.unwrap();
        assert_eq!(manifest.class_name, Some("K2SModule".to_string()));
    }

    #[test]
    fn test_manifest_without_class_name_defaults_to_none() {
        let json = r#"{
            "id": "simple-module",
            "name": "Simple Module",
            "version": "1.0.0",
            "author": "Author",
            "description": "Test module without class name",
            "urlPatterns": ["https://example.com/*"],
            "permissions": {
                "network": {
                    "allowDomains": ["example.com"]
                }
            }
        }"#;

        let manifest = ModuleManifest::from_json(json);
        assert!(manifest.is_ok());
        let manifest = manifest.unwrap();
        assert_eq!(manifest.class_name, None);
    }

    #[test]
    fn test_invalid_id_format() {
        let json = r#"{
            "id": "Test_Module",
            "name": "Test Module",
            "version": "1.0.0",
            "author": "Author",
            "description": "Test",
            "urlPatterns": ["https://example.com/*"],
            "permissions": {
                "network": {
                    "allowDomains": ["example.com"]
                }
            }
        }"#;

        let result = ModuleManifest::from_json(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("lowercase"));
    }

    #[test]
    fn test_invalid_version() {
        let json = r#"{
            "id": "test-module",
            "name": "Test Module",
            "version": "invalid",
            "author": "Author",
            "description": "Test",
            "urlPatterns": ["https://example.com/*"],
            "permissions": {
                "network": {
                    "allowDomains": ["example.com"]
                }
            }
        }"#;

        let result = ModuleManifest::from_json(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("semantic version"));
    }

    #[test]
    fn test_empty_url_patterns() {
        let json = r#"{
            "id": "test-module",
            "name": "Test Module",
            "version": "1.0.0",
            "author": "Author",
            "description": "Test",
            "urlPatterns": [],
            "permissions": {
                "network": {
                    "allowDomains": ["example.com"]
                }
            }
        }"#;

        let result = ModuleManifest::from_json(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("urlPatterns"));
    }

    #[test]
    fn test_invalid_url_pattern() {
        let json = r#"{
            "id": "test-module",
            "name": "Test Module",
            "version": "1.0.0",
            "author": "Author",
            "description": "Test",
            "urlPatterns": ["ftp://example.com/*"],
            "permissions": {
                "network": {
                    "allowDomains": ["example.com"]
                }
            }
        }"#;

        let result = ModuleManifest::from_json(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("http"));
    }

    #[test]
    fn test_empty_allow_domains() {
        let json = r#"{
            "id": "test-module",
            "name": "Test Module",
            "version": "1.0.0",
            "author": "Author",
            "description": "Test",
            "urlPatterns": ["https://example.com/*"],
            "permissions": {
                "network": {
                    "allowDomains": []
                }
            }
        }"#;

        let result = ModuleManifest::from_json(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("allowDomains"));
    }

    #[test]
    fn test_invalid_domain_pattern() {
        let json = r#"{
            "id": "test-module",
            "name": "Test Module",
            "version": "1.0.0",
            "author": "Author",
            "description": "Test",
            "urlPatterns": ["https://example.com/*"],
            "permissions": {
                "network": {
                    "allowDomains": ["invalid domain!"]
                }
            }
        }"#;

        let result = ModuleManifest::from_json(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid domain pattern"));
    }

    #[test]
    fn test_wildcard_domain_patterns() {
        let json = r#"{
            "id": "test-module",
            "name": "Test Module",
            "version": "1.0.0",
            "author": "Author",
            "description": "Test",
            "urlPatterns": ["https://example.com/*"],
            "permissions": {
                "network": {
                    "allowDomains": ["*.example.com", "api.*.example.com"]
                }
            }
        }"#;

        let result = ModuleManifest::from_json(json);
        // *.example.com should be valid
        // api.*.example.com should be invalid (wildcard not at start)
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_credential_keys() {
        let json = r#"{
            "id": "test-module",
            "name": "Test Module",
            "version": "1.0.0",
            "author": "Author",
            "description": "Test",
            "urlPatterns": ["https://example.com/*"],
            "permissions": {
                "network": {
                    "allowDomains": ["example.com"]
                },
                "credentials": {
                    "required": true,
                    "credentialKeys": []
                }
            }
        }"#;

        let result = ModuleManifest::from_json(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("credentialKeys"));
    }

    #[test]
    fn test_invalid_credential_key_format() {
        let json = r#"{
            "id": "test-module",
            "name": "Test Module",
            "version": "1.0.0",
            "author": "Author",
            "description": "Test",
            "urlPatterns": ["https://example.com/*"],
            "permissions": {
                "network": {
                    "allowDomains": ["example.com"]
                },
                "credentials": {
                    "required": true,
                    "credentialKeys": ["user-name"]
                }
            }
        }"#;

        let result = ModuleManifest::from_json(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("alphanumeric"));
    }

    #[test]
    fn test_valid_wildcard_domain() {
        assert!(ModuleManifest::is_valid_domain_pattern("*.example.com"));
        assert!(ModuleManifest::is_valid_domain_pattern("example.com"));
        assert!(ModuleManifest::is_valid_domain_pattern("api.example.com"));
    }

    #[test]
    fn test_invalid_wildcard_domain() {
        assert!(!ModuleManifest::is_valid_domain_pattern("*"));
        assert!(!ModuleManifest::is_valid_domain_pattern("example.*"));
        assert!(!ModuleManifest::is_valid_domain_pattern("*.*.example.com"));
        assert!(!ModuleManifest::is_valid_domain_pattern("example"));
        assert!(!ModuleManifest::is_valid_domain_pattern("example!.com"));
    }
}
