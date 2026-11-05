//! What a running call needs from whoever runs it: the app (`serve`, via `link::CallLink`) or
//! a module developer (`zoo-module test`, via `cli::TestHooks`).

use std::cell::RefCell;
use std::sync::Arc;

use serde::Serialize;
use tokio::sync::oneshot;
use zoo_module_client::protocol::Captcha;

use crate::budget::Budget;

pub trait Hooks {
    /// `zoo.wait`: the call is about to wait `ms`. Returns whether to really wait (replays
    /// skip countdowns).
    fn waiting(&self, ms: u64, reason: Option<String>) -> bool;
    /// Ask for a captcha solution; the receiver yields `None` if declined.
    fn ask(&self, challenge: Captcha) -> oneshot::Receiver<Option<String>>;
    /// `console.*` / `zoo.log.*`.
    fn log(&self, level: &str, message: &str);
    /// Makes the call cancellable from outside.
    fn attach(&self, _budget: Arc<Budget>) {}
}

/// What a call did, in order; shown by `zoo-module test`.
#[derive(Default)]
pub struct Trace(RefCell<Vec<Step>>);

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Step {
    Fetch { method: String, url: String, outcome: FetchOutcome, replayed: bool },
    Log { level: String, message: String },
    Wait { ms: u64, reason: Option<String>, skipped: bool },
    Captcha { challenge: String, answered: bool },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "detail", rename_all = "camelCase")]
pub enum FetchOutcome {
    Status(u16),
    /// Refused by the Policy; the message says what to change.
    Blocked(String),
    Failed(String),
}

impl Trace {
    pub fn push(&self, step: Step) {
        self.0.borrow_mut().push(step);
    }

    pub fn steps(&self) -> Vec<Step> {
        self.0.borrow().clone()
    }
}
