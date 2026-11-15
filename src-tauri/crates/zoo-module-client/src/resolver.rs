//! zoo-core's `Resolver` backed by site modules: the URL goes to the first installed module
//! whose `urlPatterns` match; other URLs download as they are.

use std::sync::{Arc, RwLock};
use std::time::Duration;

use zoo_core::resolve::{PhaseReport, Resolving};
use zoo_core::{CoreError, Phase, Resolver, Source};

use crate::manifest::url_matches;
use crate::protocol::{Captcha, Limits, ModuleError, ModuleRef};
use crate::supervisor::{Interaction, Supervisor};

/// An enabled module, as the resolver needs it.
#[derive(Debug, Clone)]
pub struct InstalledModule {
    pub module: ModuleRef,
    pub url_patterns: Vec<String>,
    /// The manifest's `urlRegex`: a URL must match it too.
    pub url_regex: Option<regex::Regex>,
    pub limits: Limits,
}

impl InstalledModule {
    pub fn matches(&self, url: &str) -> bool {
        let url = url.split('#').next().unwrap_or(url);
        self.url_patterns.iter().any(|p| url_matches(p, url)) && self.url_regex.as_ref().is_none_or(|r| r.is_match(url))
    }
}

pub struct ModuleResolver {
    supervisor: Arc<Supervisor>,
    /// In priority order: the first match resolves.
    modules: RwLock<Vec<InstalledModule>>,
    /// The user, for countdowns and captchas.
    user: Arc<dyn Interaction>,
}

impl ModuleResolver {
    pub fn new(supervisor: Arc<Supervisor>, user: Arc<dyn Interaction>) -> Self {
        Self { supervisor, modules: RwLock::new(Vec::new()), user }
    }

    /// Replace the enabled modules (after install, removal, reordering or new credentials).
    pub fn set_modules(&self, modules: Vec<InstalledModule>) {
        *self.modules.write().unwrap() = modules;
    }

    /// The module that would resolve `url`, if any.
    pub fn module_for(&self, url: &str) -> Option<InstalledModule> {
        let modules = self.modules.read().unwrap();
        modules.iter().find(|m| m.matches(url)).cloned()
    }
}

impl Resolver for ModuleResolver {
    fn resolve<'a>(&'a self, url: &'a str, report: PhaseReport) -> Resolving<'a> {
        Box::pin(async move {
            let Some(installed) = self.module_for(url) else { return Ok(Source::direct(url)) };
            let id = installed.module.id.clone();
            report(Phase::Resolving { module: Some(id.clone()) });
            let call = Reporting { user: self.user.as_ref(), report: &report, module: &id };
            let resolved = self
                .supervisor
                .resolve(installed.module, url, installed.limits, &call)
                .await
                .map_err(|e| core_error(&id, e))?;
            Ok(Source {
                url: resolved.url,
                headers: resolved.headers.into_iter().collect(),
                filename: resolved.filename,
                max_connections: resolved.max_connections.map(|n| n as usize),
                max_segment_size: resolved.max_segment_size,
                expires: true,
            })
        })
    }
}

/// Passes countdowns and captchas to the user and shows them as the task's phase.
struct Reporting<'a> {
    user: &'a dyn Interaction,
    report: &'a PhaseReport,
    module: &'a str,
}

impl Interaction for Reporting<'_> {
    fn waiting(&self, ms: u64, reason: Option<&str>) {
        let until = chrono::Utc::now() + Duration::from_millis(ms);
        (self.report)(Phase::Waiting { until, reason: reason.map(String::from) });
        self.user.waiting(ms, reason);
    }

    fn solve_captcha(
        &self,
        challenge: Captcha,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<String>> + Send + '_>> {
        Box::pin(async move {
            (self.report)(Phase::AwaitingUser);
            let solution = self.user.solve_captcha(challenge).await;
            (self.report)(Phase::Resolving { module: Some(self.module.to_string()) });
            solution
        })
    }
}

/// How a module failure shows on the task. Failures that may pass on their own (a flaky site,
/// a busy host) are retryable; a broken module is not.
fn core_error(module: &str, error: ModuleError) -> CoreError {
    let resolve = |message: String, hint: Option<String>, retryable| CoreError::Resolve { message, hint, retryable };
    match error {
        ModuleError::Cancelled => CoreError::Cancelled,
        ModuleError::Script { message, hint, .. } => resolve(format!("{module}: {message}"), hint, true),
        ModuleError::Load { message, hint } => resolve(format!("{module} is broken: {message}"), hint, false),
        ModuleError::OutOfMemory { .. } => resolve(format!("{module}: {error}"), None, false),
        ModuleError::Timeout { .. } | ModuleError::Crashed { .. } => resolve(format!("{module}: {error}"), None, true),
    }
}
