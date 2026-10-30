//! Proxy pool: the second `ConnectionSource` adapter.
//!
//! Each `acquire` hands out the next usable proxy in turn, so the segments of one download
//! spread across proxies. A failure pushes a proxy behind the healthy ones; a success
//! restores it. Only `failure_limit` consecutive failures blacklist it (until `reset_failed`).
//! With no usable proxy left, requests go direct if `fallback_direct` is on, else fail.

use crate::connection::{client_builder, ConnectionSource, Direct, Lease, Outcome};
use crate::error::CoreError;
use crate::store::JsonStore;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
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
    #[serde(default)]
    pub consecutive_failures: u32,
    pub last_verified: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    /// 0.0 - 1.0
    pub health_score: f64,
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
            consecutive_failures: 0,
            last_verified: Utc::now(),
            last_used: Utc::now(),
            health_score: 0.0,
        }
    }

    pub fn calculate_health_score(&self) -> f64 {
        let total = self.successful_requests + self.failed_requests;
        if total == 0 {
            return 0.0;
        }
        let success_rate = self.successful_requests as f64 / total as f64;
        let latency_score = 1.0 - (self.avg_latency_ms / 5000.0).min(1.0);
        let recent = Utc::now().signed_duration_since(self.last_used).num_seconds() < 3600;
        success_rate * 0.7 + latency_score * 0.2 + if recent { 0.1 } else { 0.0 }
    }

    fn url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

/// Parse one `host:port` line.
pub fn parse_proxy_line(line: &str) -> Result<ProxyInfo, String> {
    let (host, port) = match line.split(':').collect::<Vec<_>>()[..] {
        [host, port] => (host.trim(), port.trim()),
        _ => return Err(format!("invalid format '{line}', expected 'host:port'")),
    };
    if host.is_empty() {
        return Err("empty host".into());
    }
    let port = port.parse().map_err(|_| format!("invalid port number: {port}"))?;
    Ok(ProxyInfo::new(host.to_string(), port))
}

#[derive(Debug, Default, PartialEq)]
pub struct ImportResult {
    pub added: usize,
    pub duplicates: usize,
    /// `(line number, reason)` for every line that could not be parsed.
    pub invalid: Vec<(usize, String)>,
}

pub struct ProxyPool {
    state: Mutex<State>,
    store: JsonStore<Vec<ProxyInfo>>,
    direct: Direct,
    fallback_direct: AtomicBool,
    failure_limit: u32,
}

struct State {
    proxies: Vec<ProxyInfo>,
    clients: HashMap<Uuid, reqwest::Client>,
    cursor: usize,
}

impl ProxyPool {
    /// Open the pool persisted at `path` (empty if the file does not exist yet).
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, CoreError> {
        let store = JsonStore::new(path);
        let proxies = store.load()?.unwrap_or_default();
        Ok(Self {
            state: Mutex::new(State { proxies, clients: HashMap::new(), cursor: 0 }),
            store,
            direct: Direct::new()?,
            fallback_direct: AtomicBool::new(true),
            failure_limit: 5,
        })
    }

    /// Consecutive failures that blacklist a proxy.
    pub fn with_failure_limit(mut self, limit: u32) -> Self {
        self.failure_limit = limit.max(1);
        self
    }

    pub fn set_fallback_direct(&self, on: bool) {
        self.fallback_direct.store(on, Ordering::Relaxed);
    }

    /// Add proxies from `host:port` lines; `#` comments and blank lines are ignored.
    pub fn import(&self, text: &str) -> ImportResult {
        let mut result = ImportResult::default();
        let mut st = self.state.lock().unwrap();
        for (i, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            match parse_proxy_line(line) {
                Ok(p) if st.proxies.iter().any(|q| q.host == p.host && q.port == p.port) => result.duplicates += 1,
                Ok(p) => {
                    st.proxies.push(p);
                    result.added += 1;
                }
                Err(e) => result.invalid.push((i + 1, e)),
            }
        }
        self.save(&st);
        result
    }

    pub fn list(&self) -> Vec<ProxyInfo> {
        self.state.lock().unwrap().proxies.clone()
    }

    pub fn remove(&self, id: Uuid) -> Result<(), CoreError> {
        let mut st = self.state.lock().unwrap();
        let before = st.proxies.len();
        st.proxies.retain(|p| p.id != id);
        if st.proxies.len() == before {
            return Err(CoreError::NotFound);
        }
        st.clients.remove(&id);
        self.save(&st);
        Ok(())
    }

    pub fn clear(&self) {
        let mut st = self.state.lock().unwrap();
        st.proxies.clear();
        st.clients.clear();
        self.save(&st);
    }

    /// Give failed and blacklisted proxies another chance.
    pub fn reset_failed(&self) {
        let mut st = self.state.lock().unwrap();
        for p in &mut st.proxies {
            if matches!(p.status, ProxyStatus::Failed | ProxyStatus::Blacklisted) {
                p.status = ProxyStatus::Unknown;
                p.consecutive_failures = 0;
            }
        }
        self.save(&st);
    }

    /// Proxies that are not blacklisted.
    pub fn usable_count(&self) -> usize {
        self.state.lock().unwrap().proxies.iter().filter(|p| p.status != ProxyStatus::Blacklisted).count()
    }

    /// Persist current statistics (call on shutdown; routine updates are kept in memory).
    pub fn persist(&self) {
        self.save(&self.state.lock().unwrap());
    }

    fn save(&self, st: &State) {
        if let Err(e) = self.store.save(&st.proxies) {
            log::error!("failed to save {}: {e}", self.store.path().display());
        }
    }
}

impl ConnectionSource for ProxyPool {
    fn acquire(&self) -> Result<Lease, CoreError> {
        let mut st = self.state.lock().unwrap();
        // Healthy proxies first; failed ones only when nothing healthy is left.
        let healthy: Vec<usize> = (0..st.proxies.len())
            .filter(|&i| matches!(st.proxies[i].status, ProxyStatus::Active | ProxyStatus::Unknown))
            .collect();
        let candidates = if healthy.is_empty() {
            (0..st.proxies.len()).filter(|&i| st.proxies[i].status == ProxyStatus::Failed).collect()
        } else {
            healthy
        };
        if candidates.is_empty() {
            drop(st);
            return if self.fallback_direct.load(Ordering::Relaxed) {
                self.direct.acquire()
            } else {
                Err(CoreError::NoConnection("no usable proxy and direct fallback is off".into()))
            };
        }

        let index = candidates[st.cursor % candidates.len()];
        st.cursor = st.cursor.wrapping_add(1);
        let proxy = &st.proxies[index];
        let (id, url) = (proxy.id, proxy.url());
        let client = match st.clients.get(&id) {
            Some(c) => c.clone(),
            None => {
                let proxy = reqwest::Proxy::all(&url).map_err(|e| CoreError::NoConnection(format!("bad proxy {url}: {e}")))?;
                let client = client_builder()
                    .proxy(proxy)
                    .build()
                    .map_err(|e| CoreError::NoConnection(format!("cannot build client for {url}: {e}")))?;
                st.clients.insert(id, client.clone());
                client
            }
        };
        Ok(Lease { client, proxy: Some(id.to_string()) })
    }

    fn report(&self, lease: &Lease, outcome: Outcome) {
        let Some(id) = lease.proxy.as_deref().and_then(|s| Uuid::parse_str(s).ok()) else { return };
        let mut st = self.state.lock().unwrap();
        let Some(p) = st.proxies.iter_mut().find(|p| p.id == id) else { return };
        p.last_used = Utc::now();
        match outcome {
            Outcome::Success { latency } => {
                let ms = latency.as_secs_f64() * 1000.0;
                p.avg_latency_ms = if p.successful_requests == 0 { ms } else { p.avg_latency_ms * 0.7 + ms * 0.3 };
                p.successful_requests += 1;
                p.consecutive_failures = 0;
                p.status = ProxyStatus::Active;
            }
            Outcome::Failure => {
                p.failed_requests += 1;
                p.consecutive_failures += 1;
                p.status = if p.consecutive_failures >= self.failure_limit {
                    ProxyStatus::Blacklisted
                } else {
                    ProxyStatus::Failed
                };
            }
        }
        p.health_score = p.calculate_health_score();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn pool(dir: &tempfile::TempDir) -> ProxyPool {
        ProxyPool::open(dir.path().join("proxies.json")).unwrap()
    }

    fn lease_host(pool: &ProxyPool, lease: &Lease) -> Option<String> {
        let id = Uuid::parse_str(lease.proxy.as_deref()?).ok()?;
        pool.list().into_iter().find(|p| p.id == id).map(|p| p.host)
    }

    #[test]
    fn parses_lines() {
        let p = parse_proxy_line("  proxy.example.com : 3128  ").unwrap();
        assert_eq!((p.host.as_str(), p.port), ("proxy.example.com", 3128));
        for bad in ["invalid", "host:notaport", ":8080", "host:port:extra"] {
            assert!(parse_proxy_line(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn import_skips_comments_duplicates_and_invalid_lines_and_persists() {
        let dir = tempfile::tempdir().unwrap();
        let p = pool(&dir);
        let r = p.import("# list\n\na:1\nb:2\na:1\nbroken\nc:x\n");
        assert_eq!(r.added, 2);
        assert_eq!(r.duplicates, 1);
        assert_eq!(r.invalid.iter().map(|(line, _)| *line).collect::<Vec<_>>(), vec![6, 7]);

        let reopened = pool(&dir);
        assert_eq!(reopened.list().len(), 2);
    }

    #[test]
    fn acquire_rotates_across_proxies() {
        let dir = tempfile::tempdir().unwrap();
        let p = pool(&dir);
        p.import("a:1\nb:2\nc:3");
        let hosts: Vec<_> = (0..6).map(|_| lease_host(&p, &p.acquire().unwrap()).unwrap()).collect();
        assert_eq!(hosts, ["a", "b", "c", "a", "b", "c"]);
    }

    #[test]
    fn failures_deprioritize_success_restores_and_limit_blacklists() {
        let dir = tempfile::tempdir().unwrap();
        let p = pool(&dir).with_failure_limit(2);
        p.import("a:1\nb:2");
        let a = p.acquire().unwrap();
        p.report(&a, Outcome::Failure);
        // Only b is healthy now.
        for _ in 0..3 {
            assert_eq!(lease_host(&p, &p.acquire().unwrap()).as_deref(), Some("b"));
        }
        p.report(&a, Outcome::Success { latency: Duration::from_millis(10) });
        let hosts: Vec<_> = (0..2).map(|_| lease_host(&p, &p.acquire().unwrap()).unwrap()).collect();
        assert!(hosts.contains(&"a".to_string()), "a recovered after a success: {hosts:?}");

        p.report(&a, Outcome::Failure);
        p.report(&a, Outcome::Failure);
        let status = p.list().into_iter().find(|x| x.host == "a").unwrap().status;
        assert_eq!(status, ProxyStatus::Blacklisted);
        assert_eq!(p.usable_count(), 1);

        p.reset_failed();
        assert_eq!(p.usable_count(), 2);
    }

    #[test]
    fn no_usable_proxy_falls_back_to_direct_only_when_allowed() {
        let dir = tempfile::tempdir().unwrap();
        let p = pool(&dir).with_failure_limit(1);
        p.import("a:1");
        let a = p.acquire().unwrap();
        p.report(&a, Outcome::Failure);

        assert_eq!(p.acquire().unwrap().proxy, None, "fallback to direct");
        p.set_fallback_direct(false);
        assert!(matches!(p.acquire(), Err(CoreError::NoConnection(_))));
    }

    #[test]
    fn empty_pool_uses_direct() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(pool(&dir).acquire().unwrap().proxy, None);
    }

    #[test]
    fn serialized_shape_is_unchanged_for_the_frontend() {
        let json = serde_json::to_value(ProxyInfo::new("h".into(), 1)).unwrap();
        for key in ["id", "host", "port", "proxyType", "status", "avgLatencyMs", "successfulRequests", "failedRequests", "lastVerified", "lastUsed", "healthScore"] {
            assert!(json.get(key).is_some(), "missing {key}");
        }
        assert_eq!(json["status"], "unknown");
    }
}
