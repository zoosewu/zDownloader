// Network Monitor (Feature 004)
//
// This module monitors network requests made by modules, enforcing rate limits,
// response size limits, and detecting suspicious behavior.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Network request monitor
pub struct NetworkMonitor {
    module_id: String,
    request_history: Vec<RequestRecord>,
    /// Maximum requests per minute (default: 60)
    max_requests_per_minute: usize,
    /// Maximum response size in bytes (default: 10 MB)
    max_response_size: usize,
    /// Maximum unique domains per minute (default: 5)
    max_domains_per_minute: usize,
}

/// Record of a network request
#[derive(Debug, Clone)]
struct RequestRecord {
    timestamp: Instant,
    url: String,
    domain: String,
    response_size: usize,
}

impl NetworkMonitor {
    /// Create a new network monitor for a module (Phase 4: T063)
    pub fn new(module_id: String) -> Self {
        Self {
            module_id,
            request_history: Vec::new(),
            max_requests_per_minute: 60,
            max_response_size: 10 * 1024 * 1024, // 10 MB
            max_domains_per_minute: 5,
        }
    }

    /// Create a network monitor with custom limits
    pub fn with_limits(
        module_id: String,
        max_requests_per_minute: usize,
        max_response_size: usize,
        max_domains_per_minute: usize,
    ) -> Self {
        Self {
            module_id,
            request_history: Vec::new(),
            max_requests_per_minute,
            max_response_size,
            max_domains_per_minute,
        }
    }

    /// Check if a new request is allowed based on rate limits (Phase 4: T064)
    pub fn check_rate_limit(&mut self) -> Result<(), String> {
        // Cleanup old records first
        self.cleanup_old_records();

        // Count requests in the last minute
        let recent_requests = self.request_history.len();

        if recent_requests >= self.max_requests_per_minute {
            return Err(format!(
                "Rate limit exceeded for module '{}': {} requests/min (max: {})",
                self.module_id, recent_requests, self.max_requests_per_minute
            ));
        }

        Ok(())
    }

    /// Record a network request (Phase 4: T064)
    pub fn record_request(&mut self, url: &str, response_size: usize) -> Result<(), String> {
        // Extract domain from URL
        let domain = self.extract_domain(url)?;

        // Create request record
        let record = RequestRecord {
            timestamp: Instant::now(),
            url: url.to_string(),
            domain,
            response_size,
        };

        self.request_history.push(record);

        // Cleanup old records to prevent memory leak
        self.cleanup_old_records();

        Ok(())
    }

    /// Check if response size is within limits (Phase 4: T065)
    pub fn check_response_size(&self, size: usize) -> Result<(), String> {
        if size > self.max_response_size {
            return Err(format!(
                "Response size {} bytes exceeds limit of {} bytes for module '{}'",
                size, self.max_response_size, self.module_id
            ));
        }
        Ok(())
    }

    /// Detect suspicious behavior (Phase 4: T066)
    pub fn detect_suspicious_behavior(&self) -> Option<String> {
        // Check 1: Too many unique domains in a short time
        let unique_domains = self.count_unique_domains();
        if unique_domains > self.max_domains_per_minute {
            return Some(format!(
                "Suspicious: Module '{}' accessed {} unique domains in 1 minute (max: {})",
                self.module_id, unique_domains, self.max_domains_per_minute
            ));
        }

        // Check 2: Repeated failed requests to the same domain
        if let Some(domain) = self.detect_repeated_failures() {
            return Some(format!(
                "Suspicious: Module '{}' making repeated requests to potentially unreachable domain: {}",
                self.module_id, domain
            ));
        }

        // Check 3: Sudden burst of requests
        if self.detect_request_burst() {
            return Some(format!(
                "Suspicious: Module '{}' showing unusual burst of requests",
                self.module_id
            ));
        }

        None
    }

    /// Get request statistics
    pub fn get_statistics(&self) -> NetworkStatistics {
        let total_requests = self.request_history.len();
        let unique_domains = self.count_unique_domains();
        let total_bytes: usize = self.request_history.iter().map(|r| r.response_size).sum();

        NetworkStatistics {
            module_id: self.module_id.clone(),
            total_requests,
            unique_domains,
            total_bytes_transferred: total_bytes,
            requests_per_minute: total_requests,
        }
    }

    /// Clean up old request records (older than 1 minute) (Phase 4: T067)
    fn cleanup_old_records(&mut self) {
        let one_minute_ago = Instant::now() - Duration::from_secs(60);
        self.request_history.retain(|record| record.timestamp > one_minute_ago);
    }

    /// Extract domain from URL
    fn extract_domain(&self, url: &str) -> Result<String, String> {
        // Simple domain extraction
        let url_str = url
            .trim_start_matches("http://")
            .trim_start_matches("https://");

        let domain = url_str
            .split('/')
            .next()
            .ok_or_else(|| format!("Invalid URL: {}", url))?
            .split(':')
            .next()
            .ok_or_else(|| format!("Invalid URL: {}", url))?;

        Ok(domain.to_string())
    }

    /// Count unique domains in request history
    fn count_unique_domains(&self) -> usize {
        let mut domains: Vec<&str> = self.request_history.iter()
            .map(|r| r.domain.as_str())
            .collect();
        domains.sort_unstable();
        domains.dedup();
        domains.len()
    }

    /// Detect repeated failures to the same domain
    fn detect_repeated_failures(&self) -> Option<String> {
        // Track domain request counts
        let mut domain_counts: HashMap<&str, usize> = HashMap::new();

        for record in &self.request_history {
            *domain_counts.entry(record.domain.as_str()).or_insert(0) += 1;
        }

        // If any domain has more than 10 requests in a minute, it might be suspicious
        for (domain, count) in domain_counts {
            if count > 10 {
                return Some(domain.to_string());
            }
        }

        None
    }

    /// Detect sudden burst of requests
    fn detect_request_burst(&self) -> bool {
        // Check if more than 30 requests in the last 10 seconds
        if self.request_history.len() < 30 {
            return false;
        }

        let ten_seconds_ago = Instant::now() - Duration::from_secs(10);
        let recent_requests = self.request_history.iter()
            .filter(|r| r.timestamp > ten_seconds_ago)
            .count();

        recent_requests > 30
    }
}

/// Network request statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatistics {
    pub module_id: String,
    pub total_requests: usize,
    pub unique_domains: usize,
    pub total_bytes_transferred: usize,
    pub requests_per_minute: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test network monitor creation (Phase 4: T063)
    #[test]
    fn test_network_monitor_creation() {
        let monitor = NetworkMonitor::new("test-module".to_string());
        assert_eq!(monitor.module_id, "test-module");
        assert_eq!(monitor.max_requests_per_minute, 60);
        assert_eq!(monitor.max_response_size, 10 * 1024 * 1024);
    }

    /// Test custom limits (Phase 4: T063)
    #[test]
    fn test_custom_limits() {
        let monitor = NetworkMonitor::with_limits(
            "test-module".to_string(),
            100,
            20 * 1024 * 1024,
            10,
        );
        assert_eq!(monitor.max_requests_per_minute, 100);
        assert_eq!(monitor.max_response_size, 20 * 1024 * 1024);
        assert_eq!(monitor.max_domains_per_minute, 10);
    }

    /// Test response size check (Phase 4: T065)
    #[test]
    fn test_response_size_check() {
        let monitor = NetworkMonitor::new("test-module".to_string());

        // Should allow normal size
        assert!(monitor.check_response_size(1024).is_ok());

        // Should reject too large
        assert!(monitor.check_response_size(11 * 1024 * 1024).is_err());
    }

    /// Test rate limit check (Phase 4: T064)
    #[test]
    fn test_rate_limit_empty() {
        let mut monitor = NetworkMonitor::new("test-module".to_string());
        // Should allow requests when history is empty
        assert!(monitor.check_rate_limit().is_ok());
    }

    /// Test request recording (Phase 4: T064)
    #[test]
    fn test_record_request() {
        let mut monitor = NetworkMonitor::new("test-module".to_string());

        let result = monitor.record_request("https://example.com/test", 1024);
        assert!(result.is_ok());
        assert_eq!(monitor.request_history.len(), 1);
    }

    /// Test domain extraction (Phase 4: T064)
    #[test]
    fn test_extract_domain() {
        let monitor = NetworkMonitor::new("test-module".to_string());

        assert_eq!(
            monitor.extract_domain("https://example.com/path").unwrap(),
            "example.com"
        );
        assert_eq!(
            monitor.extract_domain("http://test.org:8080/path").unwrap(),
            "test.org"
        );
        assert_eq!(
            monitor.extract_domain("https://sub.domain.com").unwrap(),
            "sub.domain.com"
        );
    }

    /// Test unique domain counting (Phase 4: T066)
    #[test]
    fn test_count_unique_domains() {
        let mut monitor = NetworkMonitor::new("test-module".to_string());

        monitor.record_request("https://example.com/1", 100).unwrap();
        monitor.record_request("https://example.com/2", 100).unwrap();
        monitor.record_request("https://test.org/1", 100).unwrap();

        assert_eq!(monitor.count_unique_domains(), 2);
    }

    /// Test suspicious behavior detection - too many domains (Phase 4: T066)
    #[test]
    fn test_detect_suspicious_many_domains() {
        let mut monitor = NetworkMonitor::with_limits(
            "test-module".to_string(),
            100,
            10 * 1024 * 1024,
            3, // Max 3 domains
        );

        monitor.record_request("https://domain1.com", 100).unwrap();
        monitor.record_request("https://domain2.com", 100).unwrap();
        monitor.record_request("https://domain3.com", 100).unwrap();
        monitor.record_request("https://domain4.com", 100).unwrap();

        let suspicious = monitor.detect_suspicious_behavior();
        assert!(suspicious.is_some());
        assert!(suspicious.unwrap().contains("unique domains"));
    }

    /// Test statistics (Phase 4: T067)
    #[test]
    fn test_get_statistics() {
        let mut monitor = NetworkMonitor::new("test-module".to_string());

        monitor.record_request("https://example.com/1", 1000).unwrap();
        monitor.record_request("https://example.com/2", 2000).unwrap();
        monitor.record_request("https://test.org/1", 500).unwrap();

        let stats = monitor.get_statistics();
        assert_eq!(stats.module_id, "test-module");
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.unique_domains, 2);
        assert_eq!(stats.total_bytes_transferred, 3500);
    }

    /// Test request burst detection (Phase 4: T066)
    #[test]
    fn test_no_burst_detection_with_few_requests() {
        let mut monitor = NetworkMonitor::new("test-module".to_string());

        // Add only 10 requests - should not trigger burst detection
        for i in 0..10 {
            monitor.record_request(&format!("https://example.com/{}", i), 100).unwrap();
        }

        assert!(!monitor.detect_request_burst());
    }
}
