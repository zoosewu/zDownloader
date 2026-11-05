//! `manifest.json` of a site module, apiVersion 1.
//! The editor-facing contract is `crates/zoo-module/sdk/manifest.schema.json`; tests keep the
//! two in agreement.

use serde::{Deserialize, Serialize};

pub const API_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manifest {
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    pub api_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(default = "default_entry")]
    pub entry: String,
    pub url_patterns: Vec<String>,
    pub permissions: Permissions,
}

fn default_entry() -> String {
    "dist/index.js".into()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Permissions {
    pub network: Network,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub credentials: Vec<CredentialField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Network {
    pub allow_domains: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CredentialField {
    pub key: String,
    pub label: String,
    #[serde(default = "yes")]
    pub secret: bool,
    #[serde(default)]
    pub optional: bool,
}

fn yes() -> bool {
    true
}

impl Manifest {
    /// Parses and checks what serde cannot; errors say how to fix the file.
    pub fn parse(json: &str) -> Result<Self, String> {
        let manifest: Manifest = serde_json::from_str(json).map_err(|e| format!("manifest.json: {e}"))?;
        if manifest.api_version != API_VERSION {
            return Err(format!(
                "manifest.json: apiVersion {} is not supported; this app runs apiVersion {API_VERSION}",
                manifest.api_version
            ));
        }
        let m = &manifest;
        let id_ok = m.id.len() <= 63
            && m.id.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
            && m.id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        check(id_ok, || format!("id {:?}: use lowercase letters, digits and '-' (at most 63)", m.id))?;
        check(!m.name.is_empty() && m.name.chars().count() <= 80, || "name: 1 to 80 characters".into())?;
        check(is_semver(&m.version), || format!("version {:?}: use MAJOR.MINOR.PATCH, e.g. 1.0.0", m.version))?;
        check(m.description.as_ref().is_none_or(|d| d.chars().count() <= 500), || "description: at most 500 characters".into())?;
        check(m.author.as_ref().is_none_or(|a| a.chars().count() <= 100), || "author: at most 100 characters".into())?;
        check(m.homepage.as_ref().is_none_or(|h| h.starts_with("http://") || h.starts_with("https://")), || {
            "homepage: must be an http(s) URL".into()
        })?;
        let entry_ok = m.entry.ends_with(".js") && !m.entry.starts_with('/') && !m.entry.contains('\\');
        check(entry_ok, || format!("entry {:?}: a relative path to a .js file", m.entry))?;
        check(!m.url_patterns.is_empty(), || "urlPatterns is empty; list the share-page URLs the module handles".into())?;
        check(unique(&m.url_patterns), || "urlPatterns: duplicate entries".into())?;
        for pattern in &m.url_patterns {
            let rest = pattern.strip_prefix("https://").or_else(|| pattern.strip_prefix("http://"));
            let ok = rest.and_then(|r| r.split_once('/')).is_some_and(|(host, _)| !host.is_empty());
            check(ok, || format!("urlPatterns {pattern:?}: start with http(s)://host/, e.g. https://example.com/file/*"))?;
        }
        check(unique(&m.permissions.network.allow_domains), || "allowDomains: duplicate entries".into())?;
        for domain in &m.permissions.network.allow_domains {
            check(is_domain(domain), || {
                format!("allowDomains {domain:?}: a bare host such as example.com (no scheme, path or port)")
            })?;
        }
        for field in &m.permissions.credentials {
            let key_ok = !field.key.is_empty() && field.key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            check(key_ok, || format!("credentials key {:?}: letters, digits and '_' only", field.key))?;
            check(!field.label.is_empty(), || format!("credentials {:?}: label is empty", field.key))?;
        }
        Ok(manifest)
    }
}

/// Whether `url` matches a `urlPatterns` entry. Scheme, host and the rest are compared apart:
/// `*` matches any run of characters but never crosses from the host into the path, so
/// `https://*.k2s.cc/*` cannot match `https://evil.com/x.k2s.cc/`. Host and scheme ignore
/// case; the fragment is ignored.
pub fn url_matches(pattern: &str, url: &str) -> bool {
    let split = |s: &str| -> Option<(String, String, String)> {
        let (scheme, rest) = s.split_once("://")?;
        let (host, path) = rest.find('/').map_or((rest, "/"), |i| (&rest[..i], &rest[i..]));
        Some((scheme.to_ascii_lowercase(), host.to_ascii_lowercase(), path.to_string()))
    };
    let url = url.split('#').next().unwrap_or(url);
    match (split(pattern), split(url)) {
        (Some((ps, ph, pp)), Some((us, uh, up))) => ps == us && glob(&ph, &uh) && glob(&pp, &up),
        _ => false,
    }
}

/// `*` matches any run of characters; everything else literally (also used by test expectations).
pub fn glob(pattern: &str, text: &str) -> bool {
    let (p, t): (Vec<char>, Vec<char>) = (pattern.chars().collect(), text.chars().collect());
    let (mut pi, mut ti) = (0, 0);
    let mut backtrack: Option<(usize, usize)> = None;
    while ti < t.len() {
        if pi < p.len() && p[pi] == '*' {
            backtrack = Some((pi, ti));
            pi += 1;
        } else if pi < p.len() && p[pi] == t[ti] {
            pi += 1;
            ti += 1;
        } else if let Some((star, matched)) = backtrack {
            pi = star + 1;
            ti = matched + 1;
            backtrack = Some((star, matched + 1));
        } else {
            return false;
        }
    }
    p[pi..].iter().all(|&c| c == '*')
}

fn check(ok: bool, message: impl FnOnce() -> String) -> Result<(), String> {
    if ok {
        Ok(())
    } else {
        Err(format!("manifest.json: {}", message()))
    }
}

fn unique(items: &[String]) -> bool {
    let mut seen = std::collections::HashSet::new();
    items.iter().all(|item| seen.insert(item))
}

fn is_semver(version: &str) -> bool {
    let (core, pre) = match version.split_once('-') {
        Some((core, pre)) => (core, Some(pre)),
        None => (version, None),
    };
    let parts: Vec<&str> = core.split('.').collect();
    let number = |p: &&str| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()) && (p.len() == 1 || !p.starts_with('0'));
    parts.len() == 3
        && parts.iter().all(number)
        && pre.is_none_or(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-'))
}

fn is_domain(domain: &str) -> bool {
    let labels: Vec<&str> = domain.strip_prefix("*.").unwrap_or(domain).split('.').collect();
    labels.len() >= 2 && labels.iter().all(|l| !l.is_empty() && l.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL: &str = r#"{
        "$schema": "./types/manifest.schema.json",
        "apiVersion": 1,
        "id": "example-host",
        "name": "Example Host",
        "version": "1.2.3",
        "description": "d",
        "author": "a",
        "homepage": "https://example.com",
        "entry": "dist/index.js",
        "urlPatterns": ["https://example.com/file/*"],
        "permissions": {
            "network": { "allowDomains": ["example.com"] },
            "credentials": [{ "key": "password", "label": "Password", "secret": true, "optional": true }]
        }
    }"#;

    #[test]
    fn url_patterns() {
        let cases = [
            ("https://k2s.cc/file/*", "https://k2s.cc/file/abc/name.zip", true),
            ("https://k2s.cc/file/*", "HTTPS://K2S.CC/file/abc", true),
            ("https://k2s.cc/file/*", "https://k2s.cc/file/abc#frag", true),
            ("https://k2s.cc/file/*", "http://k2s.cc/file/abc", false),
            ("https://k2s.cc/file/*", "https://k2s.cc/folder/abc", false),
            ("https://*.k2s.cc/file/*", "https://www.k2s.cc/file/abc", true),
            ("https://*.k2s.cc/file/*", "https://k2s.cc/file/abc", false),
            ("https://*.k2s.cc/*", "https://evil.com/x.k2s.cc/file", false),
            ("https://k2s.cc/*", "https://k2s.cc.evil.com/x", false),
            ("https://site.com/f/*/d*", "https://site.com/f/1/download", true),
            ("https://site.com/f/*", "https://site.com/f/", true),
            ("https://site.com/f/*?x=1", "https://site.com/f/a?x=1", true),
        ];
        for (pattern, url, expected) in cases {
            assert_eq!(url_matches(pattern, url), expected, "{pattern} vs {url}");
        }
    }

    #[test]
    fn parses_every_field() {
        let m = Manifest::parse(FULL).unwrap();
        assert_eq!(m.permissions.credentials[0].key, "password");
        assert!(m.permissions.credentials[0].optional);
    }

    #[test]
    fn defaults_and_rejections() {
        let minimal = r#"{ "apiVersion": 1, "id": "x", "name": "X", "version": "1.0.0",
            "urlPatterns": ["https://x.com/*"], "permissions": { "network": { "allowDomains": [] },
            "credentials": [{ "key": "k", "label": "K" }] } }"#;
        let m = Manifest::parse(minimal).unwrap();
        assert_eq!(m.entry, "dist/index.js");
        assert!(m.permissions.credentials[0].secret);

        assert!(Manifest::parse(&minimal.replace("\"apiVersion\": 1", "\"apiVersion\": 2")).unwrap_err().contains("apiVersion"));
        assert!(Manifest::parse(&minimal.replace("\"id\"", "\"className\": \"X\", \"id\"")).is_err(), "unknown fields fail");
    }
}
