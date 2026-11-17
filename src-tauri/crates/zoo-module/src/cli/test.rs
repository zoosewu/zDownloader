//! `zoo-module test`: run test cases in the real host and show what happened.
//!
//! A case with a recording (tests/cassettes/<case>.json) replays offline and skips countdowns;
//! one without hits the real site. `--record` hits the real site and saves the recording.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::{BufRead, IsTerminal, Write};
use std::rc::Rc;
use std::sync::Arc;

use serde::Serialize;
use tokio::sync::oneshot;
use zoo_module_client::manifest::{glob, handles, url_regex};
use zoo_module_client::protocol::{Captcha, Limits, Listed, ModuleError, Resolved, Secret};

use super::project::{Case, Diagnostic, Project};
use crate::budget::Budget;
use crate::cassette::{self, Cassette};
use crate::hooks::{Hooks, Step, Trace};
use crate::net::{Credentials, Net, Policy, Transport};
use crate::runtime::{self, Done, Work};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    /// From the recording, offline.
    Replay,
    /// Against the real site, saving a recording.
    Record,
    /// Against the real site (no recording exists, or a URL given on the command line).
    Live,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseReport {
    pub name: String,
    pub url: String,
    pub mode: Mode,
    pub steps: Vec<Step>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved: Option<Resolved>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listed: Option<Vec<Listed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ModuleError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    pub checks: Vec<Check>,
    pub passed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    pub field: &'static str,
    pub expected: String,
    pub actual: Option<String>,
    pub ok: bool,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    /// Problems before any case could run (no manifest, no bundle, bad cases file).
    pub diagnostics: Vec<Diagnostic>,
    pub cases: Vec<CaseReport>,
    pub passed: usize,
    pub failed: usize,
}

pub struct Options {
    pub url: Option<String>,
    pub case: Option<String>,
    pub record: bool,
    pub allow_loopback: bool,
}

pub fn run(dir: &std::path::Path, options: &Options) -> Report {
    let mut report = Report::default();
    let project = match Project::open(dir).and_then(|p| p.require_entry().map(|_| p)) {
        Ok(project) => project,
        Err(diagnostic) => {
            report.diagnostics.push(diagnostic);
            return report;
        }
    };
    let credentials = match project.credentials() {
        Ok((granted, warnings)) => {
            report.diagnostics.extend(warnings);
            granted
        }
        Err(bad) => {
            report.diagnostics.push(bad);
            return report;
        }
    };
    let cases = match &options.url {
        Some(url) => vec![Case { name: "url".into(), url: url.clone(), list: false, expect: Default::default() }],
        None => match project.cases() {
            Ok(Some(cases)) => cases,
            Ok(None) => {
                report.diagnostics.push(Diagnostic::error(
                    format!("{} does not exist", project.cases_path().display()),
                    Some("add test cases, or test one URL: zoo-module test --url <share page URL>".into()),
                ));
                return report;
            }
            Err(bad) => {
                report.diagnostics.push(bad);
                return report;
            }
        },
    };
    for case in cases.iter().filter(|c| options.case.as_deref().is_none_or(|name| name == c.name)) {
        let case_report = run_case(&project, case, &credentials, options);
        if case_report.passed {
            report.passed += 1;
        } else {
            report.failed += 1;
        }
        report.cases.push(case_report);
    }
    if report.cases.is_empty() {
        report.diagnostics.push(Diagnostic::error(format!("no case named {:?}", options.case.as_deref().unwrap_or("")), None));
    }
    report
}

fn run_case(
    project: &Project,
    case: &Case,
    credentials: &std::collections::BTreeMap<String, Secret>,
    options: &Options,
) -> CaseReport {
    let cassette_path = project.cassette_path(&case.name);
    // An ad-hoc URL is never recorded or replayed.
    let adhoc = options.url.is_some();
    let recording = if options.record || adhoc { None } else { Cassette::load(&cassette_path).ok() };
    let mode = match (&recording, options.record && !adhoc) {
        (_, true) => Mode::Record,
        (Some(_), false) => Mode::Replay,
        (None, false) => Mode::Live,
    };
    let transport = match &recording {
        Some(cassette) if mode == Mode::Replay => Transport::replay(cassette),
        _ if mode == Mode::Record => Transport::Record(Default::default()),
        _ => Transport::Live,
    };
    let trace = Rc::new(Trace::default());
    let credentials = match &recording {
        Some(cassette) if mode == Mode::Replay => {
            cassette.credentials.iter().map(|key| (key.clone(), Secret("replay".into()))).collect()
        }
        _ => credentials.clone(),
    };
    let answers = recording.map(|c| c.captchas).unwrap_or_default();
    let hooks = Rc::new(TestHooks { trace: trace.clone(), mode, recorded_answers: Default::default(), replay_answers: RefCell::new(answers.into()) });

    let policy = Policy::new(&project.manifest.permissions.network.allow_domains, options.allow_loopback);
    let outcome = Net::new(policy, Credentials::new(&credentials), transport, Some(trace.clone()))
        .map_err(|message| ModuleError::Crashed { message })
        .and_then(|net| {
            let net = Rc::new(net);
            let work = if case.list { Work::List(&case.url) } else { Work::Resolve(&case.url) };
            let result = runtime::run_call(&project.entry, work, Limits::default(), net.clone(), hooks.clone());
            if mode == Mode::Record {
                let cassette = Cassette {
                    version: cassette::VERSION,
                    exchanges: net.recorded(),
                    captchas: hooks.recorded_answers.borrow().clone(),
                    credentials: credentials.keys().cloned().collect(),
                };
                if let Err(e) = cassette.save(&cassette_path) {
                    trace.push(Step::Log { level: "error".into(), message: e });
                }
            }
            result
        });

    let (resolved, listed, error) = match outcome {
        Ok(Done::Resolved(resolved)) => (Some(resolved), None, None),
        Ok(Done::Listed(items)) => (None, Some(items), None),
        Ok(Done::Loaded) => unreachable!("cases resolve or list"),
        Err(error) => (None, None, Some(error)),
    };
    let mut checks = checks(case, resolved.as_ref(), error.as_ref());
    if let Some(items) = &listed {
        checks.extend(list_checks(project, case, items));
    }
    let expected_failure = case.expect.error.is_some();
    let passed = checks.iter().all(|c| c.ok) && (error.is_some() == expected_failure);
    CaseReport {
        name: case.name.clone(),
        url: case.url.clone(),
        mode,
        steps: trace.steps(),
        hint: error.as_ref().and_then(super::hint_for),
        resolved,
        listed,
        error,
        checks,
        passed,
    }
}

/// Every listed URL must come back to this module (the app refuses the list otherwise), plus
/// the case's `count` and `files`.
fn list_checks(project: &Project, case: &Case, items: &[Listed]) -> Vec<Check> {
    let manifest = &project.manifest;
    let regex = manifest.url_regex.as_deref().and_then(|r| url_regex(r).ok());
    let mut checks: Vec<Check> = items
        .iter()
        .filter(|item| !handles(&manifest.url_patterns, regex.as_ref(), &item.url))
        .map(|item| Check { field: "listed url", expected: "matches urlPatterns".into(), actual: Some(item.url.clone()), ok: false })
        .collect();
    if let Some(count) = case.expect.count {
        checks.push(Check { field: "count", expected: count.to_string(), actual: Some(items.len().to_string()), ok: items.len() == count });
    }
    if let Some(files) = &case.expect.files {
        let path = |item: &Listed| {
            let name = item.filename.clone().unwrap_or_default();
            item.folder.as_ref().map_or(name.clone(), |folder| format!("{folder}/{name}"))
        };
        let (mut expected, mut actual) = (files.clone(), items.iter().map(path).collect::<Vec<_>>());
        expected.sort();
        actual.sort();
        let ok = expected == actual;
        checks.push(Check { field: "files", expected: expected.join(", "), actual: Some(actual.join(", ")), ok });
    }
    checks
}

fn checks(case: &Case, resolved: Option<&Resolved>, error: Option<&ModuleError>) -> Vec<Check> {
    let mut checks = Vec::new();
    let mut pattern = |field, expected: &Option<String>, actual: Option<String>| {
        if let Some(expected) = expected {
            let ok = actual.as_deref().is_some_and(|a| glob(expected, a));
            checks.push(Check { field, expected: expected.clone(), actual, ok });
        }
    };
    pattern("url", &case.expect.url, resolved.map(|r| r.url.clone()));
    pattern("filename", &case.expect.filename, resolved.and_then(|r| r.filename.clone()));
    if let Some(expected) = &case.expect.error {
        let actual = error.map(|e| e.to_string());
        let ok = actual.as_deref().is_some_and(|a| a.contains(expected.as_str()));
        checks.push(Check { field: "error", expected: expected.clone(), actual, ok });
    }
    checks
}

/// Countdowns, captchas and logs for a developer at a terminal.
struct TestHooks {
    trace: Rc<Trace>,
    mode: Mode,
    /// Answers given while recording, saved with the recording.
    recorded_answers: RefCell<Vec<Option<String>>>,
    /// Answers from the recording, used in order.
    replay_answers: RefCell<VecDeque<Option<String>>>,
}

impl Hooks for TestHooks {
    fn waiting(&self, ms: u64, reason: Option<String>) -> bool {
        let skipped = self.mode == Mode::Replay;
        if !skipped {
            eprintln!("waiting {}{}", super::seconds(ms), reason.as_deref().map(|r| format!(" ({r})")).unwrap_or_default());
        }
        self.trace.push(Step::Wait { ms, reason, skipped });
        !skipped
    }

    fn ask(&self, challenge: Captcha) -> oneshot::Receiver<Option<String>> {
        let description = match &challenge {
            Captcha::Recaptcha { site_key, page_url } => format!("reCAPTCHA site key {site_key} on {page_url}"),
            Captcha::Image { mime, .. } => format!("image captcha ({mime})"),
        };
        let answer = if self.mode == Mode::Replay {
            self.replay_answers.borrow_mut().pop_front().flatten()
        } else {
            let answer = ask_terminal(&challenge);
            if self.mode == Mode::Record {
                self.recorded_answers.borrow_mut().push(answer.clone());
            }
            answer
        };
        self.trace.push(Step::Captcha { challenge: description, answered: answer.is_some() });
        let (tx, rx) = oneshot::channel();
        let _ = tx.send(answer);
        rx
    }

    fn log(&self, level: &str, message: &str) {
        self.trace.push(Step::Log { level: level.into(), message: message.into() });
    }

    fn attach(&self, _budget: Arc<Budget>) {}
}

/// Solve a captcha by hand. Without a terminal the captcha is declined.
fn ask_terminal(challenge: &Captcha) -> Option<String> {
    if !std::io::stdin().is_terminal() {
        return None;
    }
    match challenge {
        Captcha::Image { mime, base64 } => {
            use base64::Engine;
            let ext = mime.split('/').nth(1).unwrap_or("bin").split(';').next().unwrap_or("bin");
            let path = std::env::temp_dir().join(format!("zoo-module-captcha.{ext}"));
            let bytes = base64::engine::general_purpose::STANDARD.decode(base64).ok()?;
            std::fs::write(&path, bytes).ok()?;
            eprint!("captcha image: {}\ntype the text (empty to decline): ", path.display());
        }
        Captcha::Recaptcha { site_key, page_url } => {
            eprint!(
                "reCAPTCHA (site key {site_key}) on {page_url}\nsolve it in a browser and paste the g-recaptcha-response token (empty to decline): "
            );
        }
    }
    let _ = std::io::stderr().flush();
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line).ok()?;
    Some(line.trim().to_string()).filter(|l| !l.is_empty())
}
