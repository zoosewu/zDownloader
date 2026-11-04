//! Where the bytes of a task actually come from.
//!
//! A task's `url` is what the user gave: often a share page, not the file. Before every
//! transfer a `Resolver` turns it into a `Source`: the direct URL plus the headers it needs.
//! Adapters: `Direct` (the URL is the file) and the site-module resolver in zoo-module-client.

use std::future::Future;
use std::pin::Pin;

use crate::error::CoreError;
use crate::task::Phase;

/// The direct download a task's URL resolved to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub url: String,
    /// Sent with every request of the transfer (e.g. Referer, Cookie).
    pub headers: Vec<(String, String)>,
    /// Suggested file name; used only if the user did not choose one.
    pub filename: Option<String>,
    /// Whether the link was produced by resolution (and so may expire); a direct link
    /// is never re-resolved.
    pub expires: bool,
}

impl Source {
    pub fn direct(url: &str) -> Self {
        Self { url: url.to_string(), headers: Vec::new(), filename: None, expires: false }
    }
}

/// Tells the task's owner what resolution is doing, for display.
pub type PhaseReport = Box<dyn Fn(Phase) + Send + Sync>;

pub type Resolving<'a> = Pin<Box<dyn Future<Output = Result<Source, CoreError>> + Send + 'a>>;

pub trait Resolver: Send + Sync {
    /// Resolve `url`. Dropping the future abandons the resolution.
    fn resolve<'a>(&'a self, url: &'a str, report: PhaseReport) -> Resolving<'a>;
}

/// Every URL is the file itself.
pub struct Direct;

impl Resolver for Direct {
    fn resolve<'a>(&'a self, url: &'a str, _report: PhaseReport) -> Resolving<'a> {
        Box::pin(async move { Ok(Source::direct(url)) })
    }
}
