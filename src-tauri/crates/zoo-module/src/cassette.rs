//! Recorded HTTP exchanges for `zoo-module test`: record once against the real site, replay
//! offline afterwards. One cassette per test case (`tests/cassettes/<case>.json`).
//! Credential values never appear: they are stored as `{{credential:<key>}}`.

use std::path::Path;

use base64::Engine;
use serde::{Deserialize, Serialize};

pub const VERSION: u32 = 1;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cassette {
    pub version: u32,
    /// One per HTTP hop (redirects are separate exchanges), in request order.
    pub exchanges: Vec<Exchange>,
    /// Captcha answers given while recording, in order; `null` for declined.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub captchas: Vec<Option<String>>,
    /// The credential keys the module had while recording. Replays give it the same keys
    /// (with stand-in values) whatever `.credentials.json` holds, so they take the same path.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub credentials: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exchange {
    pub method: String,
    pub url: String,
    pub status: u16,
    pub headers: Vec<(String, String)>,
    /// The body as text when it is UTF-8 ...
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// ... and as base64 otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_base64: Option<String>,
}

impl Exchange {
    pub fn set_body(&mut self, bytes: &[u8]) {
        match std::str::from_utf8(bytes) {
            Ok(text) => (self.body, self.body_base64) = (Some(text.to_string()), None),
            Err(_) => (self.body, self.body_base64) = (None, Some(base64::engine::general_purpose::STANDARD.encode(bytes))),
        }
    }

    pub fn body_bytes(&self) -> Vec<u8> {
        match (&self.body, &self.body_base64) {
            (Some(text), _) => text.as_bytes().to_vec(),
            (None, Some(b64)) => base64::engine::general_purpose::STANDARD.decode(b64).unwrap_or_default(),
            (None, None) => Vec::new(),
        }
    }
}

impl Cassette {
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let cassette: Cassette =
            serde_json::from_str(&text).map_err(|e| format!("{} is not a cassette: {e}", path.display()))?;
        if cassette.version != VERSION {
            return Err(format!("{}: cassette version {} (expected {VERSION}); record it again", path.display(), cassette.version));
        }
        Ok(cassette)
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
        }
        let text = serde_json::to_string_pretty(self).expect("cassettes serialize");
        std::fs::write(path, text + "\n").map_err(|e| format!("cannot write {}: {e}", path.display()))
    }
}
