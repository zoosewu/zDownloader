//! A module's source directory, as `zoo-module new` lays it out:
//!   manifest.json, dist/index.js (the bundle), tests/cases.json, tests/cassettes/<case>.json,
//!   .credentials.json (local only).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zoo_module_client::manifest::Manifest;
use zoo_module_client::protocol::Secret;

pub struct Project {
    pub dir: PathBuf,
    pub manifest: Manifest,
    pub entry: PathBuf,
}

/// Something `check` or `test` wants the author to know.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub level: Level,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Level {
    Error,
    Warning,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>, hint: Option<String>) -> Self {
        Self { level: Level::Error, message: message.into(), hint }
    }
    pub fn warning(message: impl Into<String>, hint: Option<String>) -> Self {
        Self { level: Level::Warning, message: message.into(), hint }
    }
}

/// One entry of tests/cases.json.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Case {
    /// Also the cassette's file name: letters, digits, '-' and '_'.
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub expect: Expect,
}

/// What the result must look like. `url` and `filename` are patterns where `*` matches any run
/// of characters; `error` is text the error message must contain (and means failure is expected).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Expect {
    pub url: Option<String>,
    pub filename: Option<String>,
    pub error: Option<String>,
}

impl Project {
    pub fn open(dir: &Path) -> Result<Self, Diagnostic> {
        let path = dir.join("manifest.json");
        let text = std::fs::read_to_string(&path).map_err(|e| {
            Diagnostic::error(
                format!("cannot read {}: {e}", path.display()),
                Some("run zoo-module inside a module directory, or create one with zoo-module new <dir>".into()),
            )
        })?;
        let manifest = Manifest::parse(&text).map_err(|message| {
            Diagnostic::error(message, Some("types/manifest.schema.json describes every field".into()))
        })?;
        let entry = dir.join(&manifest.entry);
        Ok(Self { dir: dir.to_path_buf(), manifest, entry })
    }

    /// The bundle, or a diagnostic saying how to produce it.
    pub fn require_entry(&self) -> Result<(), Diagnostic> {
        if self.entry.is_file() {
            return Ok(());
        }
        Err(Diagnostic::error(
            format!("{} does not exist", self.entry.display()),
            Some(format!("build the module first: pnpm build (bundles src/ into {})", self.manifest.entry)),
        ))
    }

    pub fn cases_path(&self) -> PathBuf {
        self.dir.join("tests").join("cases.json")
    }

    /// `None` if there is no cases file.
    pub fn cases(&self) -> Result<Option<Vec<Case>>, Diagnostic> {
        let path = self.cases_path();
        let Ok(text) = std::fs::read_to_string(&path) else { return Ok(None) };
        let cases: Vec<Case> = serde_json::from_str(&text).map_err(|e| {
            Diagnostic::error(
                format!("{}: {e}", path.display()),
                Some(r#"a JSON array of { "name": "basic", "url": "https://...", "expect": { "url": "https://cdn.example/*" } }"#.into()),
            )
        })?;
        for case in &cases {
            if case.name.is_empty() || !case.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                return Err(Diagnostic::error(
                    format!("{}: case name {:?}", path.display(), case.name),
                    Some("names become cassette file names: use letters, digits, '-' and '_'".into()),
                ));
            }
        }
        Ok(Some(cases))
    }

    pub fn cassette_path(&self, case: &str) -> PathBuf {
        self.dir.join("tests").join("cassettes").join(format!("{case}.json"))
    }

    /// `.credentials.json` (`{ "key": "value" }`), limited to the keys the manifest declares.
    /// Never commit it; it is in the template's .gitignore.
    pub fn credentials(&self) -> Result<(BTreeMap<String, Secret>, Vec<Diagnostic>), Diagnostic> {
        let path = self.dir.join(".credentials.json");
        let Ok(text) = std::fs::read_to_string(&path) else { return Ok((BTreeMap::new(), Vec::new())) };
        let values: BTreeMap<String, String> = serde_json::from_str(&text).map_err(|e| {
            Diagnostic::error(format!("{}: {e}", path.display()), Some(r#"an object such as { "username": "me", "password": "..." }"#.into()))
        })?;
        let declared: Vec<&str> = self.manifest.permissions.credentials.iter().map(|c| c.key.as_str()).collect();
        let mut warnings = Vec::new();
        let mut granted = BTreeMap::new();
        for (key, value) in values {
            if declared.contains(&key.as_str()) {
                granted.insert(key, Secret(value));
            } else {
                warnings.push(Diagnostic::warning(
                    format!(".credentials.json has {key:?}, which the manifest does not declare; it is ignored"),
                    Some(format!(r#"add {{ "key": "{key}", "label": "..." }} to permissions.credentials"#)),
                ));
            }
        }
        Ok((granted, warnings))
    }
}
