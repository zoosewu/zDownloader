// Proxy pool management - manages proxy list and health scoring
use crate::models::{ProxyInfo, ProxyStatus};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Proxy pool that manages a list of proxies with health scoring
pub struct ProxyPool {
    proxies: Arc<RwLock<Vec<ProxyInfo>>>,
    max_failures_before_blacklist: u64,
}

impl ProxyPool {
    /// Create a new empty proxy pool
    pub fn new() -> Self {
        Self {
            proxies: Arc::new(RwLock::new(Vec::new())),
            max_failures_before_blacklist: 5,
        }
    }

    /// Add multiple proxies to the pool
    pub async fn add_proxies(&self, new_proxies: Vec<ProxyInfo>) {
        let mut proxies = self.proxies.write().await;

        for proxy in new_proxies {
            // Check if proxy already exists
            if !proxies
                .iter()
                .any(|p| p.host == proxy.host && p.port == proxy.port)
            {
                proxies.push(proxy);
            }
        }
    }

    /// Remove a proxy by ID
    pub async fn remove_proxy(&self, proxy_id: Uuid) -> Result<(), String> {
        let mut proxies = self.proxies.write().await;

        let initial_len = proxies.len();
        proxies.retain(|p| p.id != proxy_id);

        if proxies.len() == initial_len {
            Err(format!("Proxy with ID {} not found", proxy_id))
        } else {
            Ok(())
        }
    }

    /// Get all proxies
    pub async fn get_all_proxies(&self) -> Vec<ProxyInfo> {
        let proxies = self.proxies.read().await;
        proxies.clone()
    }

    /// Get the best proxy based on health score
    /// Returns None if no active proxies available
    pub async fn get_best_proxy(&self) -> Option<ProxyInfo> {
        let proxies = self.proxies.read().await;

        proxies
            .iter()
            .filter(|p| p.status == ProxyStatus::Active || p.status == ProxyStatus::Unknown)
            .max_by(|a, b| {
                a.calculate_health_score()
                    .partial_cmp(&b.calculate_health_score())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    /// Mark a proxy request as successful
    pub async fn mark_success(&self, proxy_id: Uuid, latency_ms: f64) -> Result<(), String> {
        let mut proxies = self.proxies.write().await;

        if let Some(proxy) = proxies.iter_mut().find(|p| p.id == proxy_id) {
            proxy.successful_requests += 1;
            proxy.status = ProxyStatus::Active;
            proxy.last_used = chrono::Utc::now();

            // Update average latency (exponential moving average)
            if proxy.successful_requests == 1 {
                proxy.avg_latency_ms = latency_ms;
            } else {
                proxy.avg_latency_ms = proxy.avg_latency_ms * 0.7 + latency_ms * 0.3;
            }

            // Recalculate health score
            proxy.health_score = proxy.calculate_health_score();

            Ok(())
        } else {
            Err(format!("Proxy with ID {} not found", proxy_id))
        }
    }

    /// Mark a proxy request as failed
    pub async fn mark_failure(&self, proxy_id: Uuid) -> Result<(), String> {
        let mut proxies = self.proxies.write().await;

        if let Some(proxy) = proxies.iter_mut().find(|p| p.id == proxy_id) {
            proxy.failed_requests += 1;

            // Blacklist if too many failures
            if proxy.failed_requests >= self.max_failures_before_blacklist {
                proxy.status = ProxyStatus::Blacklisted;
            } else {
                proxy.status = ProxyStatus::Failed;
            }

            // Recalculate health score
            proxy.health_score = proxy.calculate_health_score();

            Ok(())
        } else {
            Err(format!("Proxy with ID {} not found", proxy_id))
        }
    }

    /// Get active proxy count
    pub async fn count_active(&self) -> usize {
        let proxies = self.proxies.read().await;
        proxies
            .iter()
            .filter(|p| p.status == ProxyStatus::Active || p.status == ProxyStatus::Unknown)
            .count()
    }

    /// Clear all proxies
    pub async fn clear(&self) {
        let mut proxies = self.proxies.write().await;
        proxies.clear()
    }

    /// Reset failed status for all proxies (useful for re-verification)
    pub async fn reset_failed_proxies(&self) {
        let mut proxies = self.proxies.write().await;

        for proxy in proxies.iter_mut() {
            if proxy.status == ProxyStatus::Failed {
                proxy.status = ProxyStatus::Unknown;
            }
        }
    }

}

impl Default for ProxyPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_proxy() {
        let pool = ProxyPool::new();
        let proxy = ProxyInfo::new("127.0.0.1".to_string(), 8080);

        pool.add_proxies(vec![proxy.clone()]).await;

        assert_eq!(pool.get_all_proxies().await.len(), 1);
        let proxies = pool.get_all_proxies().await;
        assert_eq!(proxies[0].host, "127.0.0.1");
        assert_eq!(proxies[0].port, 8080);
    }

    #[tokio::test]
    async fn test_add_duplicate_proxy() {
        let pool = ProxyPool::new();
        let proxy1 = ProxyInfo::new("127.0.0.1".to_string(), 8080);
        let proxy2 = ProxyInfo::new("127.0.0.1".to_string(), 8080);

        pool.add_proxies(vec![proxy1]).await;
        pool.add_proxies(vec![proxy2]).await;

        // Should only have one proxy (duplicates rejected)
        assert_eq!(pool.get_all_proxies().await.len(), 1);
    }

    #[tokio::test]
    async fn test_mark_success() {
        let pool = ProxyPool::new();
        let proxy = ProxyInfo::new("127.0.0.1".to_string(), 8080);
        let proxy_id = proxy.id;

        pool.add_proxies(vec![proxy]).await;
        pool.mark_success(proxy_id, 100.0).await.unwrap();

        let updated_proxy = pool.get_all_proxies().await.into_iter().find(|p| p.id == proxy_id).unwrap();
        assert_eq!(updated_proxy.successful_requests, 1);
        assert_eq!(updated_proxy.status, ProxyStatus::Active);
        assert_eq!(updated_proxy.avg_latency_ms, 100.0);
    }

    #[tokio::test]
    async fn test_mark_failure_blacklist() {
        let pool = ProxyPool::new(); // Blacklists after 5 failures
        let proxy = ProxyInfo::new("127.0.0.1".to_string(), 8080);
        let proxy_id = proxy.id;

        pool.add_proxies(vec![proxy]).await;

        // Mark as failed 5 times
        for _ in 0..5 {
            pool.mark_failure(proxy_id).await.unwrap();
        }

        let updated_proxy = pool.get_all_proxies().await.into_iter().find(|p| p.id == proxy_id).unwrap();
        assert_eq!(updated_proxy.status, ProxyStatus::Blacklisted);
        assert_eq!(updated_proxy.failed_requests, 5);
    }

    #[tokio::test]
    async fn test_get_best_proxy() {
        let pool = ProxyPool::new();

        let proxy1 = ProxyInfo::new("127.0.0.1".to_string(), 8080);
        let proxy2 = ProxyInfo::new("127.0.0.1".to_string(), 8081);
        let proxy1_id = proxy1.id;
        let proxy2_id = proxy2.id;

        pool.add_proxies(vec![proxy1]).await;
        pool.add_proxies(vec![proxy2]).await;

        // Make proxy1 better by marking it as successful
        pool.mark_success(proxy1_id, 50.0).await.unwrap();
        pool.mark_success(proxy2_id, 200.0).await.unwrap();

        let best = pool.get_best_proxy().await.unwrap();
        assert_eq!(best.id, proxy1_id); // proxy1 has lower latency
    }

}
