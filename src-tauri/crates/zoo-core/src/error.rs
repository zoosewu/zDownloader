//! Errors crossing the zoo-core interface.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "detail", rename_all = "camelCase")]
pub enum CoreError {
    /// The server answered with a non-success status.
    #[error("HTTP {0}")]
    Http(u16),
    /// Connection, DNS, TLS or timeout failure.
    #[error("network error: {0}")]
    Network(String),
    /// The server's answer does not make sense (bad Content-Range, length mismatch, ...).
    #[error("unexpected server response: {0}")]
    Protocol(String),
    /// Local filesystem failure.
    #[error("file error: {0}")]
    Io(String),
    /// No connection could be obtained (e.g. every proxy is blacklisted and direct fallback is off).
    #[error("no connection available: {0}")]
    NoConnection(String),
    #[error("cancelled")]
    Cancelled,
    #[error("task not found")]
    NotFound,
    #[error("{0}")]
    Invalid(String),
}

impl CoreError {
    /// Worth trying again later without user action.
    pub fn is_retryable(&self) -> bool {
        match self {
            CoreError::Http(code) => matches!(code, 408 | 425 | 429 | 500..=599),
            CoreError::Network(_) | CoreError::NoConnection(_) => true,
            _ => false,
        }
    }
}

impl From<std::io::Error> for CoreError {
    fn from(e: std::io::Error) -> Self {
        CoreError::Io(e.to_string())
    }
}

impl From<reqwest::Error> for CoreError {
    fn from(e: reqwest::Error) -> Self {
        match e.status() {
            Some(status) => CoreError::Http(status.as_u16()),
            None => CoreError::Network(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retryability() {
        assert!(CoreError::Http(503).is_retryable());
        assert!(CoreError::Http(429).is_retryable());
        assert!(!CoreError::Http(404).is_retryable());
        assert!(!CoreError::Http(403).is_retryable());
        assert!(CoreError::Network("reset".into()).is_retryable());
        assert!(!CoreError::Io("disk full".into()).is_retryable());
        assert!(!CoreError::Cancelled.is_retryable());
    }

    #[test]
    fn serializes_with_kind_tag() {
        let json = serde_json::to_value(CoreError::Http(404)).unwrap();
        assert_eq!(json, serde_json::json!({ "kind": "http", "detail": 404 }));
    }
}
