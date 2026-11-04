//! `zoo-module serve`: requests on stdin, replies on stdout, one JSON object per line.
//! Each call gets its own thread and V8 isolate, so a runaway module only fails its own call.
//! Stdout carries nothing but replies; diagnostics go to stderr.

use std::io::{BufRead, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{mpsc, Arc};
use std::thread;

use zoo_module_client::protocol::{ModuleError, Reply, Request};

use crate::link::{self, Answers, CallLink};
use crate::runtime::{self, HostOptions};

pub fn run(options: HostOptions) {
    let (replies, outbox) = mpsc::channel::<Reply>();
    let answers: Answers = Arc::default();
    thread::spawn(move || {
        for reply in outbox {
            let mut out = std::io::stdout().lock();
            let line = serde_json::to_string(&reply).expect("replies always serialize");
            if writeln!(out, "{line}").and_then(|_| out.flush()).is_err() {
                return; // The app is gone.
            }
        }
    });

    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<Request>(&line) {
            Ok(Request::Resolve { id, module, url, limits }) => {
                let replies = replies.clone();
                let link = CallLink::new(id, replies.clone(), answers.clone());
                thread::spawn(move || {
                    let outcome = catch_unwind(AssertUnwindSafe(|| runtime::resolve(&module, &url, limits, options, link)))
                        .unwrap_or_else(|_| Err(ModuleError::Crashed { message: "module host panicked".into() }));
                    let _ = replies.send(match outcome {
                        Ok(resolved) => Reply::Resolved { id, resolved },
                        Err(error) => Reply::Failed { id, error },
                    });
                });
            }
            Ok(Request::Answer { id, ticket, solution }) => link::answer(&answers, id, ticket, solution),
            Err(e) => eprintln!("ignoring unreadable request: {e}"),
        }
    }
    // Stdin closed: the app is gone; in-flight calls die with the process.
}
