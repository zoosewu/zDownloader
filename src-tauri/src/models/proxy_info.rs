// ProxyInfo model - represents a proxy server
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyInfo {
    pub id: Uuid,
    pub host: String,
    pub port: u16,
    pub proxy_type: ProxyType,
    pub status: ProxyStatus,
    pub avg_latency_ms: f64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub last_verified: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub health_score: f64, // 0.0 - 1.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProxyType {
    Http,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProxyStatus {
    Unknown,
    Verifying,
    Active,
    Failed,
    Blacklisted,
}

impl ProxyInfo {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            id: Uuid::new_v4(),
            host,
            port,
            proxy_type: ProxyType::Http,
            status: ProxyStatus::Unknown,
            avg_latency_ms: 0.0,
            successful_requests: 0,
            failed_requests: 0,
            last_verified: Utc::now(),
            last_used: Utc::now(),
            health_score: 0.0,
        }
    }

    pub fn calculate_health_score(&self) -> f64 {
        let total_requests = self.successful_requests + self.failed_requests;
        if total_requests == 0 {
            return 0.0;
        }

        let success_rate = self.successful_requests as f64 / total_requests as f64;
        let latency_score = 1.0 - (self.avg_latency_ms / 5000.0).min(1.0);

        let elapsed_secs = Utc::now()
            .signed_duration_since(self.last_used)
            .num_seconds();
        let recency_bonus = if elapsed_secs < 3600 { 1.0 } else { 0.0 };

        success_rate * 0.7 + latency_score * 0.2 + recency_bonus * 0.1
    }
}
