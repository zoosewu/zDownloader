//! `zoo-module check`: everything that can be verified without running `resolve`.

use std::path::Path;
use std::rc::Rc;

use zoo_module_client::protocol::{Limits, ModuleError};

use super::project::{Diagnostic, Level, Project};
use super::{hint_for, NullHooks};
use crate::net::{Credentials, Net, Policy, Transport};
use crate::runtime::{self, Work};

pub fn diagnostics(dir: &Path) -> Vec<Diagnostic> {
    let project = match Project::open(dir) {
        Ok(project) => project,
        Err(diagnostic) => return vec![diagnostic],
    };
    let mut found = Vec::new();
    if let Err(missing) = project.require_entry() {
        found.push(missing);
    } else {
        found.extend(load(&project));
        found.extend(credential_keys(&project));
    }
    match project.cases() {
        Err(bad) => found.push(bad),
        Ok(None) => found.push(Diagnostic::warning(
            format!("{} does not exist", project.cases_path().display()),
            Some("add test cases so zoo-module test can check the module".into()),
        )),
        Ok(Some(cases)) => {
            if cases.is_empty() {
                found.push(Diagnostic::warning("tests/cases.json has no cases", None));
            }
            for case in cases.iter().filter(|c| !project.cassette_path(&c.name).exists()) {
                found.push(Diagnostic::warning(
                    format!("case {:?} has no recording; zoo-module test will use the real site", case.name),
                    Some("record it once: zoo-module test --record".into()),
                ));
            }
        }
    }
    if let Err(bad) = project.credentials() {
        found.push(bad);
    }
    found
}

/// Load the bundle in the real host without calling `resolve`: syntax, imports, default export.
fn load(project: &Project) -> Option<Diagnostic> {
    let policy = Policy::new(&project.manifest.permissions.network.allow_domains, false);
    // Top-level code may not reach the network: an empty cassette refuses every request.
    let net = Net::new(policy, Credentials::new(&Default::default()), Transport::Replay(Default::default()), None).ok()?;
    match runtime::run_call(&project.entry, project.manifest.api_version, Work::Load, Limits::default(), Rc::new(net), Rc::new(NullHooks)) {
        Ok(_) => None,
        Err(error) => {
            let hint = hint_for(&error);
            let message = match error {
                ModuleError::Load { message, .. } | ModuleError::Script { message, .. } => message,
                other => other.to_string(),
            };
            Some(Diagnostic::error(format!("{}: {message}", project.manifest.entry), hint))
        }
    }
}

/// Credential keys the code uses that the manifest does not declare (they would always be
/// missing at run time).
fn credential_keys(project: &Project) -> Vec<Diagnostic> {
    let Ok(code) = std::fs::read_to_string(&project.entry) else { return Vec::new() };
    let declared: Vec<&str> = project.manifest.permissions.credentials.iter().map(|c| c.key.as_str()).collect();
    let mut used: Vec<String> = Vec::new();
    for call in ["credentials.placeholder(", "credentials.has("] {
        for (at, _) in code.match_indices(call) {
            let rest = code[at + call.len()..].trim_start();
            let Some(quote) = rest.chars().next().filter(|c| matches!(c, '\'' | '"' | '`')) else { continue };
            if let Some(end) = rest[1..].find(quote) {
                let key = rest[1..1 + end].to_string();
                if !used.contains(&key) {
                    used.push(key);
                }
            }
        }
    }
    used.into_iter()
        .filter(|key| !declared.contains(&key.as_str()))
        .map(|key| {
            Diagnostic::warning(
                format!("the code uses credential {key:?}, which manifest.json does not declare"),
                Some(format!(r#"add {{ "key": "{key}", "label": "..." }} to permissions.credentials"#)),
            )
        })
        .collect()
}

pub fn has_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(|d| d.level == Level::Error)
}
