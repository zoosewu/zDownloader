// URL Validator (Feature 004)
//
// This module validates URLs to prevent SSRF attacks, including:
// - Private/internal IP detection
// - DNS rebinding protection
// - Protocol validation

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};

/// URL validator with SSRF protection
pub struct UrlValidator {
    /// Whether to allow localhost (127.0.0.1)
    allow_localhost: bool,
    /// Whether to allow private IPs (10.x, 192.168.x, etc.)
    allow_private_ips: bool,
    /// List of allowed protocols
    allowed_protocols: Vec<String>,
}

/// Validation result
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Valid,
    InvalidProtocol(String),
    PrivateIp(String),
    LocalhostBlocked,
    DnsRebinding(String),
    InvalidUrl(String),
}

impl Default for UrlValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl UrlValidator {
    /// Create a new URL validator with default settings (Phase 4: T068)
    pub fn new() -> Self {
        Self {
            allow_localhost: false,
            allow_private_ips: false,
            allowed_protocols: vec!["http".to_string(), "https".to_string()],
        }
    }

    /// Create a URL validator with custom settings
    pub fn with_settings(
        allow_localhost: bool,
        allow_private_ips: bool,
        allowed_protocols: Vec<String>,
    ) -> Self {
        Self {
            allow_localhost,
            allow_private_ips,
            allowed_protocols,
        }
    }

    /// Validate a URL for SSRF protection (Phase 4: T068-T072)
    pub fn validate(&self, url: &str) -> Result<(), String> {
        // Step 1: Protocol check (T071)
        self.validate_protocol(url)?;

        // Step 2: Parse URL and extract host
        let host = self.extract_host(url)?;

        // Step 3: Private IP detection (T069)
        if let Ok(ip_addr) = host.parse::<IpAddr>() {
            return self.validate_ip_address(&ip_addr);
        }

        // Step 4: DNS resolution and validation (T070)
        self.validate_dns(&host)?;

        Ok(())
    }

    /// Validate protocol (Phase 4: T071)
    fn validate_protocol(&self, url: &str) -> Result<(), String> {
        let protocol = url
            .split("://")
            .next()
            .ok_or_else(|| "Invalid URL: missing protocol".to_string())?
            .to_lowercase();

        if !self.allowed_protocols.contains(&protocol) {
            return Err(format!(
                "Invalid protocol '{}'. Allowed: {:?}",
                protocol, self.allowed_protocols
            ));
        }

        Ok(())
    }

    /// Extract host from URL
    fn extract_host(&self, url: &str) -> Result<String, String> {
        let url_without_protocol = url
            .trim_start_matches("http://")
            .trim_start_matches("https://");

        let host = url_without_protocol
            .split('/')
            .next()
            .ok_or_else(|| "Invalid URL: cannot extract host".to_string())?
            .split(':')
            .next()
            .ok_or_else(|| "Invalid URL: cannot extract host".to_string())?;

        Ok(host.to_string())
    }

    /// Validate IP address for private/localhost (Phase 4: T069)
    fn validate_ip_address(&self, ip: &IpAddr) -> Result<(), String> {
        match ip {
            IpAddr::V4(ipv4) => self.validate_ipv4(ipv4),
            IpAddr::V6(ipv6) => self.validate_ipv6(ipv6),
        }
    }

    /// Validate IPv4 address (Phase 4: T069)
    fn validate_ipv4(&self, ip: &Ipv4Addr) -> Result<(), String> {
        // Check localhost (127.0.0.0/8)
        if ip.is_loopback() {
            if !self.allow_localhost {
                return Err("Localhost access blocked".to_string());
            }
            return Ok(());
        }

        // Check private IP ranges (Phase 4: T069)
        if ip.is_private() {
            if !self.allow_private_ips {
                return Err(format!("Private IP address blocked: {}", ip));
            }
            return Ok(());
        }

        // Check link-local (169.254.0.0/16)
        if ip.is_link_local() {
            return Err(format!("Link-local IP address blocked: {}", ip));
        }

        // Check broadcast
        if ip.is_broadcast() {
            return Err("Broadcast IP address blocked".to_string());
        }

        // Check multicast
        if ip.is_multicast() {
            return Err("Multicast IP address blocked".to_string());
        }

        Ok(())
    }

    /// Validate IPv6 address (Phase 4: T069)
    fn validate_ipv6(&self, ip: &Ipv6Addr) -> Result<(), String> {
        // Check localhost (::1)
        if ip.is_loopback() {
            if !self.allow_localhost {
                return Err("Localhost access blocked (IPv6)".to_string());
            }
            return Ok(());
        }

        // Check multicast
        if ip.is_multicast() {
            return Err("Multicast IPv6 address blocked".to_string());
        }

        // Check unique local (fc00::/7) - similar to private IPv4
        if is_ipv6_unique_local(ip) {
            if !self.allow_private_ips {
                return Err(format!("Private IPv6 address blocked: {}", ip));
            }
            return Ok(());
        }

        Ok(())
    }

    /// Validate DNS resolution (Phase 4: T070 - DNS rebinding protection)
    fn validate_dns(&self, host: &str) -> Result<(), String> {
        // Resolve DNS
        let socket_addr = format!("{}:80", host);
        let mut addresses = socket_addr
            .to_socket_addrs()
            .map_err(|e| format!("DNS resolution failed for '{}': {}", host, e))?;

        // Check all resolved IPs
        while let Some(addr) = addresses.next() {
            let ip = addr.ip();

            // Validate each resolved IP (Phase 4: T070 - prevent DNS rebinding)
            if let Err(e) = self.validate_ip_address(&ip) {
                return Err(format!(
                    "DNS rebinding detected: '{}' resolves to blocked IP {}: {}",
                    host, ip, e
                ));
            }
        }

        Ok(())
    }

    /// Quick validation without DNS resolution (for performance)
    pub fn quick_validate(&self, url: &str) -> Result<(), String> {
        // Only check protocol and format, skip DNS
        self.validate_protocol(url)?;
        let _host = self.extract_host(url)?;
        Ok(())
    }
}

/// Check if IPv6 address is unique local (fc00::/7)
fn is_ipv6_unique_local(ip: &Ipv6Addr) -> bool {
    let segments = ip.segments();
    (segments[0] & 0xfe00) == 0xfc00
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test URL validator creation (Phase 4: T068)
    #[test]
    fn test_url_validator_creation() {
        let validator = UrlValidator::new();
        assert!(!validator.allow_localhost);
        assert!(!validator.allow_private_ips);
        assert_eq!(validator.allowed_protocols.len(), 2);
    }

    /// Test protocol validation (Phase 4: T071)
    #[test]
    fn test_protocol_validation() {
        let validator = UrlValidator::new();

        // Valid protocols
        assert!(validator.validate_protocol("http://example.com").is_ok());
        assert!(validator.validate_protocol("https://example.com").is_ok());

        // Invalid protocols
        assert!(validator.validate_protocol("ftp://example.com").is_err());
        assert!(validator.validate_protocol("file:///etc/passwd").is_err());
        assert!(validator.validate_protocol("javascript:alert(1)").is_err());
    }

    /// Test host extraction (Phase 4: T068)
    #[test]
    fn test_extract_host() {
        let validator = UrlValidator::new();

        assert_eq!(
            validator.extract_host("http://example.com/path").unwrap(),
            "example.com"
        );
        assert_eq!(
            validator.extract_host("https://test.org:8080/path").unwrap(),
            "test.org"
        );
        assert_eq!(
            validator.extract_host("http://192.168.1.1").unwrap(),
            "192.168.1.1"
        );
    }

    /// Test IPv4 localhost detection (Phase 4: T069)
    #[test]
    fn test_ipv4_localhost_blocking() {
        let validator = UrlValidator::new();
        let localhost = "127.0.0.1".parse::<Ipv4Addr>().unwrap();

        let result = validator.validate_ipv4(&localhost);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Localhost"));
    }

    /// Test IPv4 localhost allowed (Phase 4: T069)
    #[test]
    fn test_ipv4_localhost_allowed() {
        let validator = UrlValidator::with_settings(true, false, vec!["http".to_string()]);
        let localhost = "127.0.0.1".parse::<Ipv4Addr>().unwrap();

        let result = validator.validate_ipv4(&localhost);
        assert!(result.is_ok());
    }

    /// Test IPv4 private IP detection (Phase 4: T069)
    #[test]
    fn test_ipv4_private_ip_blocking() {
        let validator = UrlValidator::new();

        // Test 10.0.0.0/8
        let ip1 = "10.0.0.1".parse::<Ipv4Addr>().unwrap();
        assert!(validator.validate_ipv4(&ip1).is_err());

        // Test 192.168.0.0/16
        let ip2 = "192.168.1.1".parse::<Ipv4Addr>().unwrap();
        assert!(validator.validate_ipv4(&ip2).is_err());

        // Test 172.16.0.0/12
        let ip3 = "172.16.0.1".parse::<Ipv4Addr>().unwrap();
        assert!(validator.validate_ipv4(&ip3).is_err());
    }

    /// Test IPv4 public IP allowed (Phase 4: T069)
    #[test]
    fn test_ipv4_public_ip_allowed() {
        let validator = UrlValidator::new();

        // Test public IP (Google DNS)
        let ip = "8.8.8.8".parse::<Ipv4Addr>().unwrap();
        assert!(validator.validate_ipv4(&ip).is_ok());
    }

    /// Test IPv4 link-local blocking (Phase 4: T069)
    #[test]
    fn test_ipv4_link_local_blocking() {
        let validator = UrlValidator::new();
        let link_local = "169.254.1.1".parse::<Ipv4Addr>().unwrap();

        let result = validator.validate_ipv4(&link_local);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Link-local"));
    }

    /// Test IPv6 localhost blocking (Phase 4: T069)
    #[test]
    fn test_ipv6_localhost_blocking() {
        let validator = UrlValidator::new();
        let localhost = "::1".parse::<Ipv6Addr>().unwrap();

        let result = validator.validate_ipv6(&localhost);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Localhost"));
    }

    /// Test IPv6 unique local blocking (Phase 4: T069)
    #[test]
    fn test_ipv6_unique_local_blocking() {
        let validator = UrlValidator::new();
        let unique_local = "fc00::1".parse::<Ipv6Addr>().unwrap();

        let result = validator.validate_ipv6(&unique_local);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Private IPv6"));
    }

    /// Test is_ipv6_unique_local helper (Phase 4: T069)
    #[test]
    fn test_is_ipv6_unique_local() {
        let unique_local = "fc00::1".parse::<Ipv6Addr>().unwrap();
        assert!(is_ipv6_unique_local(&unique_local));

        let public = "2001:4860:4860::8888".parse::<Ipv6Addr>().unwrap();
        assert!(!is_ipv6_unique_local(&public));
    }

    /// Test quick validation (Phase 4: T068)
    #[test]
    fn test_quick_validate() {
        let validator = UrlValidator::new();

        // Should pass quick validation (no DNS resolution)
        assert!(validator.quick_validate("https://example.com").is_ok());
        assert!(validator.quick_validate("http://192.168.1.1").is_ok());

        // Should fail on protocol
        assert!(validator.quick_validate("ftp://example.com").is_err());
    }

    /// Test custom allowed protocols (Phase 4: T071)
    #[test]
    fn test_custom_allowed_protocols() {
        let validator = UrlValidator::with_settings(
            false,
            false,
            vec!["http".to_string(), "https".to_string(), "ftp".to_string()],
        );

        assert!(validator.validate_protocol("ftp://example.com").is_ok());
    }

    /// Test IP address validation (Phase 4: T069)
    #[test]
    fn test_validate_ip_address() {
        let validator = UrlValidator::new();

        // Test IPv4 public
        let ip1 = "8.8.8.8".parse::<IpAddr>().unwrap();
        assert!(validator.validate_ip_address(&ip1).is_ok());

        // Test IPv4 private
        let ip2 = "192.168.1.1".parse::<IpAddr>().unwrap();
        assert!(validator.validate_ip_address(&ip2).is_err());

        // Test IPv6 public
        let ip3 = "2001:4860:4860::8888".parse::<IpAddr>().unwrap();
        assert!(validator.validate_ip_address(&ip3).is_ok());

        // Test IPv6 localhost
        let ip4 = "::1".parse::<IpAddr>().unwrap();
        assert!(validator.validate_ip_address(&ip4).is_err());
    }

    /// Test broadcast IP blocking (Phase 4: T069)
    #[test]
    fn test_broadcast_ip_blocking() {
        let validator = UrlValidator::new();
        let broadcast = "255.255.255.255".parse::<Ipv4Addr>().unwrap();

        let result = validator.validate_ipv4(&broadcast);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Broadcast"));
    }
}
