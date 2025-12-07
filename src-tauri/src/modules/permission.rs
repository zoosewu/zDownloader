// Module Permission System (Feature 004)
//
// This module implements the permission system for checking network access,
// credential access, and enforcing domain whitelists.

use super::manifest::ModulePermissions;
use wildmatch::WildMatch;

/// Permission checker for module operations
#[derive(Debug)]
pub struct PermissionChecker {
    permissions: ModulePermissions,
    domain_matchers: Vec<WildMatch>,
}

impl PermissionChecker {
    /// Create a new permission checker with module permissions
    pub fn new(permissions: ModulePermissions) -> Self {
        // Pre-compile wildmatch patterns for better performance
        let domain_matchers = permissions
            .network
            .allow_domains
            .iter()
            .map(|pattern| WildMatch::new(pattern))
            .collect();

        Self {
            permissions,
            domain_matchers,
        }
    }

    /// Check if a network request to the given URL is allowed
    pub fn check_network_permission(&self, url: &str) -> Result<(), String> {
        // Extract domain from URL
        let domain = Self::extract_domain(url)?;

        // Check if domain matches any whitelist pattern
        for matcher in self.domain_matchers.iter() {
            if matcher.matches(&domain) {
                return Ok(());
            }
        }

        Err(format!(
            "Permission denied: Domain '{}' is not in the whitelist: {:?}",
            domain, self.permissions.network.allow_domains
        ))
    }

    /// Check if credential access is allowed
    pub fn check_credential_permission(&self) -> Result<(), String> {
        if self.permissions.credentials.is_some() {
            Ok(())
        } else {
            Err("Module does not have credential permission".to_string())
        }
    }

    /// Extract domain from URL
    fn extract_domain(url: &str) -> Result<String, String> {
        // Simple domain extraction (full URL parsing would use url crate)
        let url_lower = url.to_lowercase();

        // Remove protocol
        let without_protocol = if let Some(idx) = url_lower.find("://") {
            &url_lower[idx + 3..]
        } else {
            &url_lower
        };

        // Extract domain (before first / or :)
        let domain = without_protocol
            .split(|c| c == '/' || c == ':')
            .next()
            .ok_or_else(|| format!("Invalid URL: {}", url))?;

        if domain.is_empty() {
            return Err(format!("Invalid URL: {}", url));
        }

        Ok(domain.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::manifest::{NetworkPermission, CredentialPermission};

    fn create_test_permissions(domains: Vec<String>) -> ModulePermissions {
        ModulePermissions {
            network: NetworkPermission {
                allow_domains: domains,
            },
            credentials: None,
        }
    }

    #[test]
    fn test_permission_creation() {
        let perms = create_test_permissions(vec!["example.com".to_string()]);
        let checker = PermissionChecker::new(perms);
        // Should create without errors
    }

    #[test]
    fn test_exact_domain_match() {
        let perms = create_test_permissions(vec!["example.com".to_string()]);
        let checker = PermissionChecker::new(perms);

        let result = checker.check_network_permission("https://example.com/path");
        assert!(result.is_ok(), "Should allow exact domain match");
    }

    #[test]
    fn test_wildcard_domain_match() {
        let perms = create_test_permissions(vec!["*.example.com".to_string()]);
        let checker = PermissionChecker::new(perms);

        let result = checker.check_network_permission("https://api.example.com/path");
        assert!(result.is_ok(), "Should allow wildcard domain match");

        let result = checker.check_network_permission("https://cdn.example.com/file.js");
        assert!(result.is_ok(), "Should allow wildcard subdomain");
    }

    #[test]
    fn test_domain_not_in_whitelist() {
        let perms = create_test_permissions(vec!["example.com".to_string()]);
        let checker = PermissionChecker::new(perms);

        let result = checker.check_network_permission("https://evil.com/path");
        assert!(result.is_err(), "Should deny domain not in whitelist");
    }

    #[test]
    fn test_multiple_domains() {
        let perms = create_test_permissions(vec![
            "example.com".to_string(),
            "*.cdn.com".to_string(),
            "api.service.io".to_string(),
        ]);
        let checker = PermissionChecker::new(perms);

        assert!(checker.check_network_permission("https://example.com/").is_ok());
        assert!(checker.check_network_permission("https://files.cdn.com/").is_ok());
        assert!(checker.check_network_permission("https://api.service.io/").is_ok());
        assert!(checker.check_network_permission("https://hacker.com/").is_err());
    }

    #[test]
    fn test_credential_permission() {
        let mut perms = create_test_permissions(vec!["example.com".to_string()]);
        perms.credentials = Some(CredentialPermission {
            required: true,
            credential_keys: vec!["username".to_string(), "password".to_string()],
        });

        let checker = PermissionChecker::new(perms);
        assert!(checker.check_credential_permission().is_ok());
    }

    #[test]
    fn test_no_credential_permission() {
        let perms = create_test_permissions(vec!["example.com".to_string()]);
        let checker = PermissionChecker::new(perms);

        let result = checker.check_credential_permission();
        assert!(result.is_err(), "Should deny credential access when not configured");
    }
}
