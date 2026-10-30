//! Where HTTP connections come from.
//!
//! Every transfer worker asks a `ConnectionSource` for a client before each request and
//! reports how it went. Adapters: `Direct` (one shared client) and the proxy pool (Phase 2d).

use crate::error::CoreError;
use std::time::Duration;

pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
/// Longest gap allowed between two reads before a connection counts as stalled.
pub const READ_TIMEOUT: Duration = Duration::from_secs(60);
const USER_AGENT: &str = concat!("zoo-downloader/", env!("CARGO_PKG_VERSION"));

/// A client to use for one request, plus what the source needs to score the outcome.
#[derive(Clone)]
pub struct Lease {
    pub client: reqwest::Client,
    /// Which proxy served this lease, if any.
    pub proxy: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outcome {
    Success { latency: Duration },
    Failure,
}

pub trait ConnectionSource: Send + Sync {
    fn acquire(&self) -> Result<Lease, CoreError>;
    fn report(&self, lease: &Lease, outcome: Outcome);
}

/// Client settings shared by every adapter. No total timeout: bodies may stream for hours.
pub fn client_builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .read_timeout(READ_TIMEOUT)
        .user_agent(USER_AGENT)
}

/// Connect straight to the origin.
pub struct Direct {
    client: reqwest::Client,
}

impl Direct {
    pub fn new() -> Result<Self, CoreError> {
        let client = client_builder()
            .build()
            .map_err(|e| CoreError::Network(format!("cannot build HTTP client: {e}")))?;
        Ok(Self { client })
    }
}

impl ConnectionSource for Direct {
    fn acquire(&self) -> Result<Lease, CoreError> {
        Ok(Lease { client: self.client.clone(), proxy: None })
    }

    fn report(&self, _: &Lease, _: Outcome) {}
}
