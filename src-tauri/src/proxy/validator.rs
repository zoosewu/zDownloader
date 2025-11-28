// Proxy validator - batch validation of proxy servers
use crate::models::{ProxyInfo, ProxyStatus};
use futures::stream::{self, StreamExt};
use reqwest::Client;
use std::time::Duration;

/// Batch size for concurrent proxy validation
const BATCH_SIZE: usize = 50;

/// Timeout for each proxy validation (5 seconds)
const VALIDATION_TIMEOUT: Duration = Duration::from_secs(5);

/// URL to use for proxy validation testing
const TEST_URL: &str = "http://www.google.com";

/// Result of proxy validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub proxy_id: uuid::Uuid,
    pub is_valid: bool,
    pub latency_ms: Option<f64>,
    pub error_message: Option<String>,
}

/// Validate a list of proxies in batches
///
/// # Arguments
/// * `proxies` - List of proxies to validate
///
/// # Returns
/// Vector of ValidationResult for each proxy
pub async fn validate_proxies_batch(proxies: Vec<ProxyInfo>) -> Vec<ValidationResult> {
    let mut all_results = Vec::new();

    // Process proxies in batches
    for chunk in proxies.chunks(BATCH_SIZE) {
        let batch_results = validate_batch(chunk.to_vec()).await;
        all_results.extend(batch_results);
    }

    all_results
}

/// Validate a single batch of proxies concurrently
async fn validate_batch(proxies: Vec<ProxyInfo>) -> Vec<ValidationResult> {
    // Create a stream of validation tasks
    let tasks = stream::iter(proxies)
        .map(|proxy| validate_single_proxy(proxy))
        .buffer_unordered(BATCH_SIZE);

    // Collect all results
    tasks.collect::<Vec<_>>().await
}

/// Validate a single proxy by attempting to make a test request
async fn validate_single_proxy(proxy: ProxyInfo) -> ValidationResult {
    let start_time = std::time::Instant::now();

    // Build proxy URL
    let proxy_url = format!("http://{}:{}", proxy.host, proxy.port);

    // Try to create a reqwest client with the proxy
    let client_result = Client::builder()
        .proxy(
            reqwest::Proxy::http(&proxy_url)
                .unwrap_or_else(|_| reqwest::Proxy::http("http://invalid").unwrap()),
        )
        .timeout(VALIDATION_TIMEOUT)
        .build();

    let client = match client_result {
        Ok(c) => c,
        Err(e) => {
            return ValidationResult {
                proxy_id: proxy.id,
                is_valid: false,
                latency_ms: None,
                error_message: Some(format!("Failed to create client: {}", e)),
            };
        }
    };

    // Make a test request
    match client.get(TEST_URL).send().await {
        Ok(response) => {
            let elapsed = start_time.elapsed();
            let latency_ms = elapsed.as_millis() as f64;

            if response.status().is_success() {
                ValidationResult {
                    proxy_id: proxy.id,
                    is_valid: true,
                    latency_ms: Some(latency_ms),
                    error_message: None,
                }
            } else {
                ValidationResult {
                    proxy_id: proxy.id,
                    is_valid: false,
                    latency_ms: Some(latency_ms),
                    error_message: Some(format!("HTTP error: {}", response.status())),
                }
            }
        }
        Err(e) => ValidationResult {
            proxy_id: proxy.id,
            is_valid: false,
            latency_ms: None,
            error_message: Some(format!("Request failed: {}", e)),
        },
    }
}

/// Validate and update proxy statuses in the pool
///
/// This is a helper function that validates proxies and returns updated ProxyInfo
pub async fn validate_and_update_proxies(proxies: Vec<ProxyInfo>) -> Vec<ProxyInfo> {
    let validation_results = validate_proxies_batch(proxies.clone()).await;

    // Update proxy info based on validation results
    proxies
        .into_iter()
        .zip(validation_results.into_iter())
        .map(|(mut proxy, result)| {
            if result.is_valid {
                proxy.status = ProxyStatus::Active;
                if let Some(latency) = result.latency_ms {
                    proxy.avg_latency_ms = latency;
                }
                proxy.last_verified = chrono::Utc::now();
            } else {
                proxy.status = ProxyStatus::Failed;
                proxy.last_verified = chrono::Utc::now();
            }
            proxy
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_single_invalid_proxy() {
        // Test with an invalid proxy (unlikely to exist)
        let proxy = ProxyInfo::new("192.0.2.1".to_string(), 9999); // TEST-NET-1 address

        let result = validate_single_proxy(proxy.clone()).await;

        assert_eq!(result.proxy_id, proxy.id);
        assert!(!result.is_valid);
        assert!(result.error_message.is_some());
    }

    #[tokio::test]
    async fn test_validate_batch_multiple_proxies() {
        // Create multiple invalid proxies
        let proxies = vec![
            ProxyInfo::new("192.0.2.1".to_string(), 8080),
            ProxyInfo::new("192.0.2.2".to_string(), 8080),
            ProxyInfo::new("192.0.2.3".to_string(), 8080),
        ];

        let results = validate_batch(proxies.clone()).await;

        assert_eq!(results.len(), 3);
        for result in results {
            assert!(!result.is_valid);
        }
    }

    #[tokio::test]
    async fn test_validate_proxies_batch_chunking() {
        // Create more than BATCH_SIZE proxies to test chunking
        let mut proxies = Vec::new();
        for i in 0..60 {
            proxies.push(ProxyInfo::new(format!("192.0.2.{}", i % 254 + 1), 8080));
        }

        let results = validate_proxies_batch(proxies.clone()).await;

        // Should process all proxies
        assert_eq!(results.len(), 60);
    }

    #[tokio::test]
    async fn test_validate_and_update_proxies() {
        let proxies = vec![
            ProxyInfo::new("192.0.2.1".to_string(), 8080),
            ProxyInfo::new("192.0.2.2".to_string(), 8080),
        ];

        let updated_proxies = validate_and_update_proxies(proxies.clone()).await;

        assert_eq!(updated_proxies.len(), 2);
        for proxy in updated_proxies {
            // All should be marked as Failed since they're invalid
            assert_eq!(proxy.status, ProxyStatus::Failed);
        }
    }
}
