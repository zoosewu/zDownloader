// Health scoring utilities and configuration
use crate::models::ProxyInfo;

/// Health score thresholds for proxy categorization
#[derive(Debug, Clone, Copy)]
pub struct HealthThresholds {
    pub excellent: f64, // >= 0.8
    pub good: f64,      // >= 0.6
    pub fair: f64,      // >= 0.4
    pub poor: f64,      // < 0.4
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            excellent: 0.8,
            good: 0.6,
            fair: 0.4,
            poor: 0.0,
        }
    }
}

/// Health category for a proxy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthCategory {
    Excellent, // >= 0.8
    Good,      // >= 0.6
    Fair,      // >= 0.4
    Poor,      // < 0.4
}

impl HealthCategory {
    /// Get a human-readable description
    pub fn description(&self) -> &str {
        match self {
            HealthCategory::Excellent => "Excellent - Very fast and reliable",
            HealthCategory::Good => "Good - Fast and mostly reliable",
            HealthCategory::Fair => "Fair - Acceptable but not optimal",
            HealthCategory::Poor => "Poor - Slow or unreliable",
        }
    }
}

/// Categorize a proxy based on its health score
pub fn categorize_health(health_score: f64, thresholds: &HealthThresholds) -> HealthCategory {
    if health_score >= thresholds.excellent {
        HealthCategory::Excellent
    } else if health_score >= thresholds.good {
        HealthCategory::Good
    } else if health_score >= thresholds.fair {
        HealthCategory::Fair
    } else {
        HealthCategory::Poor
    }
}

/// Get health category for a proxy using default thresholds
pub fn get_proxy_health_category(proxy: &ProxyInfo) -> HealthCategory {
    let thresholds = HealthThresholds::default();
    categorize_health(proxy.health_score, &thresholds)
}

/// Filter proxies by minimum health score
pub fn filter_by_min_health(proxies: Vec<ProxyInfo>, min_health: f64) -> Vec<ProxyInfo> {
    proxies
        .into_iter()
        .filter(|p| p.calculate_health_score() >= min_health)
        .collect()
}

/// Filter proxies by health category
pub fn filter_by_category(
    proxies: Vec<ProxyInfo>,
    min_category: HealthCategory,
) -> Vec<ProxyInfo> {
    let thresholds = HealthThresholds::default();
    let min_score = match min_category {
        HealthCategory::Excellent => thresholds.excellent,
        HealthCategory::Good => thresholds.good,
        HealthCategory::Fair => thresholds.fair,
        HealthCategory::Poor => thresholds.poor,
    };

    filter_by_min_health(proxies, min_score)
}

/// Statistics about proxy health distribution
#[derive(Debug, Clone)]
pub struct HealthStats {
    pub total: usize,
    pub excellent: usize,
    pub good: usize,
    pub fair: usize,
    pub poor: usize,
    pub average_score: f64,
}

/// Calculate health statistics for a list of proxies
pub fn calculate_health_stats(proxies: &[ProxyInfo]) -> HealthStats {
    let thresholds = HealthThresholds::default();
    let mut stats = HealthStats {
        total: proxies.len(),
        excellent: 0,
        good: 0,
        fair: 0,
        poor: 0,
        average_score: 0.0,
    };

    if proxies.is_empty() {
        return stats;
    }

    let mut total_score = 0.0;

    for proxy in proxies {
        let score = proxy.calculate_health_score();
        total_score += score;

        let category = categorize_health(score, &thresholds);
        match category {
            HealthCategory::Excellent => stats.excellent += 1,
            HealthCategory::Good => stats.good += 1,
            HealthCategory::Fair => stats.fair += 1,
            HealthCategory::Poor => stats.poor += 1,
        }
    }

    stats.average_score = total_score / proxies.len() as f64;
    stats
}

/// Recommend whether to use a proxy based on its health score
pub fn should_use_proxy(proxy: &ProxyInfo, min_acceptable_score: f64) -> bool {
    proxy.calculate_health_score() >= min_acceptable_score
}

/// Get the best N proxies by health score
pub fn get_best_n_proxies(mut proxies: Vec<ProxyInfo>, n: usize) -> Vec<ProxyInfo> {
    proxies.sort_by(|a, b| {
        b.calculate_health_score()
            .partial_cmp(&a.calculate_health_score())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    proxies.into_iter().take(n).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_proxy(success: u64, failures: u64, latency: f64) -> ProxyInfo {
        let mut proxy = ProxyInfo::new("127.0.0.1".to_string(), 8080);
        proxy.successful_requests = success;
        proxy.failed_requests = failures;
        proxy.avg_latency_ms = latency;
        proxy.last_used = chrono::Utc::now();
        proxy.health_score = proxy.calculate_health_score();
        proxy
    }

    #[test]
    fn test_categorize_health() {
        let thresholds = HealthThresholds::default();

        assert_eq!(
            categorize_health(0.9, &thresholds),
            HealthCategory::Excellent
        );
        assert_eq!(categorize_health(0.7, &thresholds), HealthCategory::Good);
        assert_eq!(categorize_health(0.5, &thresholds), HealthCategory::Fair);
        assert_eq!(categorize_health(0.3, &thresholds), HealthCategory::Poor);
    }

    #[test]
    fn test_get_proxy_health_category() {
        // Create a proxy with high success rate and low latency
        let good_proxy = create_test_proxy(90, 10, 100.0);
        let category = get_proxy_health_category(&good_proxy);

        // Should be at least Good
        assert!(
            category == HealthCategory::Excellent || category == HealthCategory::Good
        );
    }

    #[test]
    fn test_filter_by_min_health() {
        let proxies = vec![
            create_test_proxy(100, 0, 50.0),  // Excellent
            create_test_proxy(70, 30, 100.0), // Good
            create_test_proxy(50, 50, 200.0), // Fair
            create_test_proxy(10, 90, 500.0), // Poor
        ];

        let filtered = filter_by_min_health(proxies.clone(), 0.6);

        // Count how many proxies actually have health >= 0.6
        let expected_count = proxies
            .iter()
            .filter(|p| p.calculate_health_score() >= 0.6)
            .count();

        assert_eq!(filtered.len(), expected_count);

        // Verify all filtered proxies meet the threshold
        for proxy in filtered {
            assert!(proxy.calculate_health_score() >= 0.6);
        }
    }

    #[test]
    fn test_calculate_health_stats() {
        let proxies = vec![
            create_test_proxy(100, 0, 50.0),
            create_test_proxy(70, 30, 100.0),
            create_test_proxy(50, 50, 200.0),
        ];

        let stats = calculate_health_stats(&proxies);

        assert_eq!(stats.total, 3);
        assert!(stats.average_score > 0.0);
        assert!(stats.average_score <= 1.0);
    }

    #[test]
    fn test_get_best_n_proxies() {
        let proxies = vec![
            create_test_proxy(100, 0, 50.0),  // Best
            create_test_proxy(70, 30, 100.0), // Middle
            create_test_proxy(50, 50, 200.0), // Worst
        ];

        let best_2 = get_best_n_proxies(proxies, 2);

        assert_eq!(best_2.len(), 2);
        // First should be better than second
        assert!(best_2[0].calculate_health_score() >= best_2[1].calculate_health_score());
    }

    #[test]
    fn test_should_use_proxy() {
        let good_proxy = create_test_proxy(90, 10, 100.0);
        let poor_proxy = create_test_proxy(10, 90, 500.0);

        assert!(should_use_proxy(&good_proxy, 0.5));
        assert!(!should_use_proxy(&poor_proxy, 0.5));
    }

    #[test]
    fn test_health_stats_empty() {
        let proxies: Vec<ProxyInfo> = vec![];
        let stats = calculate_health_stats(&proxies);

        assert_eq!(stats.total, 0);
        assert_eq!(stats.average_score, 0.0);
    }
}
