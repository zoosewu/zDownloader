//! A running call's line to the app: events out, captcha answers back.

use std::cell::Cell;
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};

use tokio::sync::oneshot;
use zoo_module_client::protocol::{CallEvent, Captcha, Reply};

/// Answers the app has yet to give, by (call id, ticket). Shared by every call in the host.
pub type Answers = Arc<Mutex<HashMap<(u64, u64), oneshot::Sender<Option<String>>>>>;

pub struct CallLink {
    id: u64,
    replies: mpsc::Sender<Reply>,
    answers: Answers,
    next_ticket: Cell<u64>,
}

impl CallLink {
    pub fn new(id: u64, replies: mpsc::Sender<Reply>, answers: Answers) -> Self {
        Self { id, replies, answers, next_ticket: Cell::new(1) }
    }

    pub fn event(&self, event: CallEvent) {
        let _ = self.replies.send(Reply::Event { id: self.id, event });
    }

    /// Sends the challenge; the receiver yields the user's solution (None if declined, or if
    /// the app went away).
    pub fn ask(&self, challenge: Captcha) -> oneshot::Receiver<Option<String>> {
        let ticket = self.next_ticket.get();
        self.next_ticket.set(ticket + 1);
        let (tx, rx) = oneshot::channel();
        self.answers.lock().unwrap().insert((self.id, ticket), tx);
        self.event(CallEvent::Captcha { ticket, challenge });
        rx
    }
}

impl Drop for CallLink {
    fn drop(&mut self) {
        self.answers.lock().unwrap().retain(|(id, _), _| *id != self.id);
    }
}

/// Delivers an answer from the app to the call waiting for it.
pub fn answer(answers: &Answers, id: u64, ticket: u64, solution: Option<String>) {
    if let Some(waiter) = answers.lock().unwrap().remove(&(id, ticket)) {
        let _ = waiter.send(solution);
    }
}
