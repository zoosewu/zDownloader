//! A running call's line to the app: events out, captcha answers back.

use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::sync::{mpsc, Arc, Mutex};

use tokio::sync::oneshot;
use zoo_module_client::protocol::{CallEvent, Captcha, Reply};

use crate::budget::Budget;
use crate::hooks::Hooks;

/// Where a captcha answer goes, by (call id, ticket).
type AnswerSlots = HashMap<(u64, u64), oneshot::Sender<Option<String>>>;

/// What the host tracks for calls in flight, shared by every call.
#[derive(Clone, Default)]
pub struct Calls {
    /// Answers the app has yet to give, by (call id, ticket).
    answers: Arc<Mutex<AnswerSlots>>,
    /// Running calls' budgets, by call id, so the app can cancel them.
    budgets: Arc<Mutex<HashMap<u64, Arc<Budget>>>>,
    /// Cancelled before their isolate existed.
    early_cancels: Arc<Mutex<HashSet<u64>>>,
}

impl Calls {
    /// Delivers an answer from the app to the call waiting for it.
    pub fn answer(&self, id: u64, ticket: u64, solution: Option<String>) {
        if let Some(waiter) = self.answers.lock().unwrap().remove(&(id, ticket)) {
            let _ = waiter.send(solution);
        }
    }

    pub fn cancel(&self, id: u64) {
        match self.budgets.lock().unwrap().remove(&id) {
            Some(budget) => budget.cancel(),
            None => {
                self.early_cancels.lock().unwrap().insert(id);
            }
        }
        self.answers.lock().unwrap().retain(|(call, _), _| *call != id);
    }
}

pub struct CallLink {
    id: u64,
    replies: mpsc::Sender<Reply>,
    calls: Calls,
    next_ticket: Cell<u64>,
}

impl CallLink {
    pub fn new(id: u64, replies: mpsc::Sender<Reply>, calls: Calls) -> Self {
        Self { id, replies, calls, next_ticket: Cell::new(1) }
    }

    fn event(&self, event: CallEvent) {
        let _ = self.replies.send(Reply::Event { id: self.id, event });
    }
}

/// Events go to the app; logs to stderr, which the app's Supervisor logs.
impl Hooks for CallLink {
    fn waiting(&self, ms: u64, reason: Option<String>) -> bool {
        self.event(CallEvent::Waiting { ms, reason });
        true
    }

    /// Sends the challenge; the receiver yields the user's solution (None if declined, or if
    /// the app went away).
    fn ask(&self, challenge: Captcha) -> oneshot::Receiver<Option<String>> {
        let ticket = self.next_ticket.get();
        self.next_ticket.set(ticket + 1);
        let (tx, rx) = oneshot::channel();
        self.calls.answers.lock().unwrap().insert((self.id, ticket), tx);
        self.event(CallEvent::Captcha { ticket, challenge });
        rx
    }

    fn log(&self, level: &str, message: &str) {
        eprintln!("[call {}] [{level}] {message}", self.id);
    }

    /// Makes the call cancellable through `Calls::cancel`.
    fn attach(&self, budget: Arc<Budget>) {
        if self.calls.early_cancels.lock().unwrap().remove(&self.id) {
            budget.cancel();
        } else {
            self.calls.budgets.lock().unwrap().insert(self.id, budget);
        }
    }
}

impl Drop for CallLink {
    fn drop(&mut self) {
        self.calls.answers.lock().unwrap().retain(|(id, _), _| *id != self.id);
        self.calls.budgets.lock().unwrap().remove(&self.id);
    }
}
