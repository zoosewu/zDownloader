//! `zoo-marketplace.json`: what a marketplace offers. Unknown fields are ignored, so a catalog
//! written for a newer app still lists its modules here (docs/architecture/module-versions.md).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const FILE: &str = "zoo-marketplace.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    /// Identifies the marketplace: modules are `id@name`.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub modules: Vec<CatalogModule>,
}

/// One module a marketplace offers. Its URL patterns and permissions come from its own
/// manifest.json, never from here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CatalogModule {
    pub id: String,
    pub version: String,
    /// The module's directory, relative to the catalog (holds manifest.json and the entry).
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// SHA-256 of the entry (the bundled JS), lowercase hex. Required for remote marketplaces.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// The manifest's `apiVersion`, so an app can tell it needs a newer version of itself
    /// without downloading the module. Must match the manifest when given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_version: Option<u32>,
    /// Set by the app when it lists the catalog (whatever the file says): this app is too old
    /// for the module.
    #[serde(default)]
    pub needs_newer_app: bool,
}

impl CatalogModule {
    /// Why this app cannot run the listed version, if it cannot.
    pub fn unsupported(&self) -> Option<String> {
        self.api_version.and_then(|level| zoo_module_client::manifest::check_api_version(level).err())
    }
}

impl Catalog {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let catalog: Catalog = serde_json::from_slice(bytes).map_err(|e| format!("{FILE}: {e}"))?;
        if !is_name(&catalog.name) {
            return Err(format!("{FILE}: name {:?}: use lowercase letters, digits and '-'", catalog.name));
        }
        let mut seen = std::collections::HashSet::new();
        for m in &catalog.modules {
            if !seen.insert(&m.id) {
                return Err(format!("{FILE}: module {:?} is listed twice", m.id));
            }
            if !is_relative(&m.source) {
                return Err(format!("{FILE}: module {:?}: source {:?} must be a relative path inside the marketplace", m.id, m.source));
            }
            if m.sha256.as_ref().is_some_and(|h| h.len() != 64 || !h.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))) {
                return Err(format!("{FILE}: module {:?}: sha256 must be 64 lowercase hex digits", m.id));
            }
        }
        Ok(catalog)
    }

    pub fn module(&self, id: &str) -> Option<&CatalogModule> {
        self.modules.iter().find(|m| m.id == id)
    }
}

pub fn is_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 63
        && name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !name.starts_with('-')
}

/// A path that stays inside its base: `/`-separated, no `..`, not absolute. `.` is the base.
pub fn is_relative(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.contains(':')
        && path.split('/').all(|part| !part.is_empty() && part != "..")
}

/// Whether version `a` is newer than `b` (MAJOR.MINOR.PATCH, a release is newer than its
/// pre-releases). Unparsable versions are never newer.
pub fn is_newer(a: &str, b: &str) -> bool {
    fn parse(v: &str) -> Option<([u64; 3], bool)> {
        let (core, pre) = v.split_once('-').map_or((v, false), |(c, _)| (c, true));
        let mut parts = core.split('.').map(|p| p.parse::<u64>().ok());
        let version = [parts.next()??, parts.next()??, parts.next()??];
        Some((version, !pre))
    }
    matches!((parse(a), parse(b)), (Some(a), Some(b)) if a > b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_names_paths_and_hashes() {
        let ok = br#"{ "name": "zoo", "modules": [{ "id": "k2s", "version": "1.0.0", "source": "modules/k2s" }] }"#;
        assert_eq!(Catalog::parse(ok).unwrap().module("k2s").unwrap().source, "modules/k2s");
        for bad in [
            r#"{ "name": "Zoo", "modules": [] }"#,
            r#"{ "name": "zoo", "modules": [{ "id": "a", "version": "1.0.0", "source": "../x" }] }"#,
            r#"{ "name": "zoo", "modules": [{ "id": "a", "version": "1.0.0", "source": "/abs" }] }"#,
            r#"{ "name": "zoo", "modules": [{ "id": "a", "version": "1.0.0", "source": "a", "sha256": "XYZ" }] }"#,
            r#"{ "name": "zoo", "modules": [{ "id": "a", "version": "1.0.0", "source": "a" }, { "id": "a", "version": "1.0.0", "source": "b" }] }"#,
        ] {
            assert!(Catalog::parse(bad.as_bytes()).is_err(), "{bad}");
        }
        assert!(is_relative(".") && is_relative("a/b") && !is_relative("a//b") && !is_relative("C:/x"));
    }

    #[test]
    fn compares_versions() {
        assert!(is_newer("1.0.1", "1.0.0"));
        assert!(is_newer("1.10.0", "1.9.9"));
        assert!(is_newer("1.0.0", "1.0.0-beta"));
        assert!(!is_newer("1.0.0", "1.0.0"));
        assert!(!is_newer("0.9.0", "1.0.0"));
        assert!(!is_newer("x", "1.0.0"));
    }

    #[test]
    fn fields_for_newer_apps_are_ignored() {
        let catalog = Catalog::parse(br#"{ "name": "m", "futureField": 1, "modules": [
            { "id": "a", "version": "2.0.0", "source": "a", "apiVersion": 9, "releases": [] }
        ] }"#)
        .unwrap();
        let a = catalog.module("a").unwrap();
        assert!(a.unsupported().unwrap().contains("needs a newer Zoo Downloader"));
    }
}
