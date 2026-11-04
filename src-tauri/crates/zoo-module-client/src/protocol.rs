//! Wire format between the app and `zoo-module serve`: one JSON object per line.
//! App → host on stdin: `Request`. Host → app on stdout: `Reply`. Host diagnostics go to stderr.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// An installed module: its entry point (the bundled single-file ESM) and what the user let it do.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleRef {
    /// `id@marketplace`; used in logs and errors.
    pub id: String,
    pub entry: PathBuf,
    #[serde(default)]
    pub grants: Grants,
}

/// Permissions the user consented to (from the manifest). Everything not granted is denied.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Grants {
    /// Hosts `fetch` may reach; each entry also covers its subdomains.
    pub allow_domains: Vec<String>,
    /// Credential values by key (`[A-Za-z0-9_]+`). Module code only ever sees placeholders.
    #[serde(default)]
    pub credentials: BTreeMap<String, Secret>,
}

/// A credential value. Never printed.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Secret(pub String);

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret(***)")
    }
}

/// Resource limits for one `resolve` call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Limits {
    pub heap_mb: u32,
    /// Wall-clock budget for the call.
    pub timeout_ms: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self { heap_mb: 128, timeout_ms: 30_000 }
    }
}

/// What a module's `resolve(url, zoo)` returns: the direct download.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Resolved {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ModuleError {
    /// The module file could not be read, parsed or linked, or has the wrong shape.
    #[error("module load failed: {message}")]
    Load { message: String, hint: Option<String> },
    /// `resolve` threw, rejected, or returned something invalid.
    #[error("{message}")]
    Script { message: String, hint: Option<String>, stack: Option<String> },
    #[error("module exceeded its {limit_ms} ms time limit")]
    Timeout { limit_ms: u64 },
    #[error("module exceeded its {limit_mb} MB memory limit")]
    OutOfMemory { limit_mb: u32 },
    /// The host process died, could not start, or stopped answering.
    #[error("module host failed: {message}")]
    Crashed { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Request {
    Resolve { id: u64, module: ModuleRef, url: String, limits: Limits },
    /// The user's answer to a `CallEvent::Captcha`; `None` if they declined.
    Answer { id: u64, ticket: u64, solution: Option<String> },
}

impl Request {
    pub fn id(&self) -> u64 {
        match self {
            Request::Resolve { id, .. } | Request::Answer { id, .. } => *id,
        }
    }
}

/// `id` is the call's id. A call gets any number of `Event`s, then one `Resolved` or `Failed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Reply {
    Event { id: u64, event: CallEvent },
    Resolved { id: u64, resolved: Resolved },
    Failed { id: u64, error: ModuleError },
}

impl Reply {
    pub fn id(&self) -> u64 {
        match self {
            Reply::Event { id, .. } | Reply::Resolved { id, .. } | Reply::Failed { id, .. } => *id,
        }
    }
}

/// Something the user should see while a call runs. Neither counts against the time limit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum CallEvent {
    /// `zoo.wait`: the site makes free users wait.
    Waiting { ms: u64, reason: Option<String> },
    /// The module needs a captcha solved; answer with `Request::Answer` and this ticket.
    Captcha { ticket: u64, challenge: Captcha },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Captcha {
    /// Solve in a webview showing `page_url`; the solution is the g-recaptcha-response token.
    Recaptcha { site_key: String, page_url: String },
    /// Show the image (already fetched through the module's session); the solution is the text.
    Image { mime: String, base64: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn request_wire_shape() {
        let req = Request::Resolve {
            id: 7,
            module: ModuleRef {
                id: "k2s@official".into(),
                entry: "m/index.js".into(),
                grants: Grants {
                    allow_domains: vec!["k2s.cc".into()],
                    credentials: BTreeMap::from([("password".into(), Secret("hunter2".into()))]),
                },
            },
            url: "https://example.com/f".into(),
            limits: Limits::default(),
        };
        assert_eq!(
            serde_json::to_value(&req).unwrap(),
            json!({
                "type": "resolve",
                "id": 7,
                "module": { "id": "k2s@official", "entry": "m/index.js", "grants": { "allowDomains": ["k2s.cc"], "credentials": { "password": "hunter2" } } },
                "url": "https://example.com/f",
                "limits": { "heapMb": 128, "timeoutMs": 30000 }
            })
        );
    }

    #[test]
    fn reply_wire_shape_and_roundtrip() {
        let replies = [
            Reply::Resolved {
                id: 1,
                resolved: Resolved {
                    url: "https://cdn/x".into(),
                    filename: Some("x.zip".into()),
                    headers: BTreeMap::from([("Referer".into(), "https://a".into())]),
                    size: Some(10),
                },
            },
            Reply::Failed { id: 2, error: ModuleError::Timeout { limit_ms: 5 } },
            Reply::Failed {
                id: 3,
                error: ModuleError::Script { message: "m".into(), hint: Some("h".into()), stack: None },
            },
        ];
        for reply in &replies {
            let text = serde_json::to_string(reply).unwrap();
            assert_eq!(&serde_json::from_str::<Reply>(&text).unwrap(), reply);
        }
        assert_eq!(
            serde_json::to_value(&replies[1]).unwrap(),
            json!({ "type": "failed", "id": 2, "error": { "kind": "timeout", "limitMs": 5 } })
        );
    }

    #[test]
    fn secrets_never_print() {
        let grants = Grants { allow_domains: vec![], credentials: BTreeMap::from([("p".into(), Secret("hunter2".into()))]) };
        assert!(!format!("{grants:?}").contains("hunter2"));
    }

    #[test]
    fn event_wire_shape() {
        let reply = Reply::Event {
            id: 4,
            event: CallEvent::Captcha {
                ticket: 1,
                challenge: Captcha::Recaptcha { site_key: "k".into(), page_url: "https://p".into() },
            },
        };
        assert_eq!(
            serde_json::to_value(&reply).unwrap(),
            json!({ "type": "event", "id": 4, "event": { "kind": "captcha", "ticket": 1,
                    "challenge": { "type": "recaptcha", "siteKey": "k", "pageUrl": "https://p" } } })
        );
        let answer = Request::Answer { id: 4, ticket: 1, solution: None };
        assert_eq!(serde_json::to_value(&answer).unwrap(), json!({ "type": "answer", "id": 4, "ticket": 1, "solution": null }));
    }

    #[test]
    fn resolved_rejects_unknown_fields() {
        // Typos like `fileName` must fail loudly instead of being dropped.
        assert!(serde_json::from_value::<Resolved>(json!({ "url": "u", "fileName": "x" })).is_err());
        let min: Resolved = serde_json::from_value(json!({ "url": "u" })).unwrap();
        assert_eq!(min.filename, None);
        assert!(min.headers.is_empty());
    }
}
