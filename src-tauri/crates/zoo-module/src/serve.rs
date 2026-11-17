//! `zoo-module serve`: requests on stdin, replies on stdout, one JSON object per line.
//! Each call gets its own thread and V8 isolate, so a runaway module only fails its own call.
//! Stdout carries nothing but replies; diagnostics go to stderr.

use std::io::{BufRead, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc;
use std::thread;

use zoo_module_client::protocol::{Limits, ModuleError, ModuleRef, Reply, Request};

use crate::link::{CallLink, Calls};
use crate::runtime::{self, Done, HostOptions, Work};

pub fn run(options: HostOptions) {
    let (replies, outbox) = mpsc::channel::<Reply>();
    let calls = Calls::default();
    thread::spawn(move || {
        for reply in outbox {
            let mut out = std::io::stdout().lock();
            let line = serde_json::to_string(&reply).expect("replies always serialize");
            if writeln!(out, "{line}").and_then(|_| out.flush()).is_err() {
                return; // The app is gone.
            }
        }
    });

    // Each call on its own thread, replying `Resolved` or `Listed` (`list`) or `Failed`.
    let start = |id: u64, module: ModuleRef, url: String, limits: Limits, list: bool| {
        let replies = replies.clone();
        let link = CallLink::new(id, replies.clone(), calls.clone());
        thread::spawn(move || {
            let work = if list { Work::List(&url) } else { Work::Resolve(&url) };
            let outcome = catch_unwind(AssertUnwindSafe(|| runtime::call(&module, work, limits, options, link)))
                .unwrap_or_else(|_| Err(ModuleError::Crashed { message: "module host panicked".into() }));
            let _ = replies.send(match outcome {
                Ok(Done::Resolved(resolved)) => Reply::Resolved { id, resolved },
                Ok(Done::Listed(items)) => Reply::Listed { id, items },
                Ok(Done::Loaded) => unreachable!("serve calls always resolve or list"),
                Err(error) => Reply::Failed { id, error },
            });
        });
    };

    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<Request>(&line) {
            Ok(Request::Resolve { id, module, url, limits }) => start(id, module, url, limits, false),
            Ok(Request::List { id, module, url, limits }) => start(id, module, url, limits, true),
            Ok(Request::Answer { id, ticket, solution }) => calls.answer(id, ticket, solution),
            Ok(Request::Cancel { id }) => calls.cancel(id),
            Err(e) => eprintln!("ignoring unreadable request: {e}"),
        }
    }
    // Stdin closed: the app is gone; in-flight calls die with the process.
}
