//! A call's time budget. It runs while the module computes or waits on the network, and is
//! paused while the module waits for the user or a site countdown (`zoo.wait`, captchas).
//! When it runs out, a watchdog thread terminates the isolate (stopping synchronous loops) and
//! wakes `expired()` (stopping waits on host work).

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use deno_core::v8;

pub const TRIP_NONE: u8 = 0;
pub const TRIP_TIMEOUT: u8 = 1;
pub const TRIP_MEMORY: u8 = 2;

pub struct Budget {
    state: Mutex<State>,
    wake: Condvar,
    expired: tokio::sync::Notify,
}

struct State {
    used: Duration,
    /// None while paused (nested pauses count up `paused`).
    running_since: Option<Instant>,
    paused: u32,
    finished: bool,
}

impl Budget {
    /// Starts running immediately. `tripped` records why the call ended early.
    pub fn start(limit: Duration, isolate: v8::IsolateHandle, tripped: Arc<AtomicU8>) -> Arc<Self> {
        let budget = Arc::new(Self {
            state: Mutex::new(State { used: Duration::ZERO, running_since: Some(Instant::now()), paused: 0, finished: false }),
            wake: Condvar::new(),
            expired: tokio::sync::Notify::new(),
        });
        let watched = budget.clone();
        thread::spawn(move || watched.watch(limit, isolate, tripped));
        budget
    }

    fn watch(&self, limit: Duration, isolate: v8::IsolateHandle, tripped: Arc<AtomicU8>) {
        let mut state = self.state.lock().unwrap();
        loop {
            if state.finished {
                return;
            }
            match state.running_since {
                None => state = self.wake.wait(state).unwrap(),
                Some(since) => {
                    let used = state.used + since.elapsed();
                    if used >= limit {
                        let _ = tripped.compare_exchange(TRIP_NONE, TRIP_TIMEOUT, Ordering::SeqCst, Ordering::SeqCst);
                        isolate.terminate_execution();
                        self.expired.notify_one();
                        return;
                    }
                    state = self.wake.wait_timeout(state, limit - used).unwrap().0;
                }
            }
        }
    }

    /// Stops the clock until the returned guard is dropped.
    pub fn pause(self: &Arc<Self>) -> Paused {
        let mut state = self.state.lock().unwrap();
        if state.paused == 0 {
            if let Some(since) = state.running_since.take() {
                state.used += since.elapsed();
            }
        }
        state.paused += 1;
        self.wake.notify_all();
        Paused(self.clone())
    }

    /// Completes once the budget has run out.
    pub async fn expired(&self) {
        self.expired.notified().await;
    }

    /// Disarms the watchdog.
    pub fn finish(&self) {
        self.state.lock().unwrap().finished = true;
        self.wake.notify_all();
    }
}

pub struct Paused(Arc<Budget>);

impl Drop for Paused {
    fn drop(&mut self) {
        let mut state = self.0.state.lock().unwrap();
        state.paused -= 1;
        if state.paused == 0 && !state.finished {
            state.running_since = Some(Instant::now());
        }
        self.0.wake.notify_all();
    }
}
