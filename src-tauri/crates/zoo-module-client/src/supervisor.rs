//! Owns the `zoo-module serve` process: starts it on first use, multiplexes concurrent calls
//! over its stdio, and replaces it when it exits or stops answering. A dying host fails only
//! the calls in flight (as `ModuleError::Crashed`); the next call starts a fresh one.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};
use tokio::sync::oneshot;

use crate::protocol::{Limits, ModuleError, ModuleRef, Reply, Request, Resolved};

/// How long past a call's own time limit we wait before declaring the host hung.
const RESPONSE_GRACE: Duration = Duration::from_secs(2);
/// Whole-process cap (Windows Job Object); V8's per-call heap limit is the primary guard.
#[cfg(windows)]
const PROCESS_MEMORY_LIMIT: usize = 2 << 30;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub struct Supervisor {
    program: PathBuf,
    args: Vec<String>,
    host: tokio::sync::Mutex<Option<Host>>,
    next_id: AtomicU64,
    spawns: AtomicU64,
}

impl Supervisor {
    /// `program args…` must speak the protocol on stdio; in the app that is `zoo-module serve`.
    pub fn new(program: impl Into<PathBuf>, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            host: tokio::sync::Mutex::new(None),
            next_id: AtomicU64::new(1),
            spawns: AtomicU64::new(0),
        }
    }

    pub async fn resolve(&self, module: ModuleRef, url: &str, limits: Limits) -> Result<Resolved, ModuleError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (generation, reply) = self.send(Request::Resolve { id, module, url: url.into(), limits }).await?;
        let wait = Duration::from_millis(limits.timeout_ms) + RESPONSE_GRACE;
        match tokio::time::timeout(wait, reply).await {
            Ok(Ok(Reply::Resolved { resolved, .. })) => Ok(resolved),
            Ok(Ok(Reply::Failed { error, .. })) => Err(error),
            Ok(Err(_)) => Err(crashed("module host exited")),
            Err(_) => {
                self.kill(generation).await;
                Err(crashed("module host stopped responding"))
            }
        }
    }

    /// How many host processes have been started; more than one means restarts happened.
    pub fn spawn_count(&self) -> u64 {
        self.spawns.load(Ordering::Relaxed)
    }

    async fn send(&self, request: Request) -> Result<(u64, oneshot::Receiver<Reply>), ModuleError> {
        let mut line = serde_json::to_string(&request).expect("requests always serialize");
        line.push('\n');
        let mut slot = self.host.lock().await;
        // A host found dead (or dying between the check and registration) is replaced once.
        for _ in 0..2 {
            if let Some(host) = slot.as_mut() {
                if let Some(reply) = host.register(request.id()) {
                    if let Err(e) = host.stdin.write_all(line.as_bytes()).await {
                        *slot = None;
                        return Err(crashed(format!("cannot write to module host: {e}")));
                    }
                    return Ok((host.generation, reply));
                }
            }
            *slot = Some(self.spawn()?);
        }
        Err(crashed("module host exited during startup"))
    }

    fn spawn(&self) -> Result<Host, ModuleError> {
        let mut command = Command::new(&self.program);
        command
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        #[cfg(windows)]
        command.creation_flags(CREATE_NO_WINDOW);
        let mut child = command
            .spawn()
            .map_err(|e| crashed(format!("cannot start {}: {e}", self.program.display())))?;

        #[cfg(windows)]
        let job = child.raw_handle().and_then(|process| {
            crate::job::Job::assign(process, PROCESS_MEMORY_LIMIT)
                .map_err(|e| log::warn!(target: "zoo_module", "module host runs without a job object: {e}"))
                .ok()
        });

        let pending = Arc::new(Mutex::new(Pending { open: true, waiters: HashMap::new() }));
        tokio::spawn(read_replies(child.stdout.take().expect("piped"), pending.clone()));
        tokio::spawn(forward_stderr(child.stderr.take().expect("piped")));
        Ok(Host {
            generation: self.spawns.fetch_add(1, Ordering::Relaxed) + 1,
            stdin: child.stdin.take().expect("piped"),
            pending,
            _child: child,
            #[cfg(windows)]
            _job: job,
        })
    }

    /// Kill the host if it is still the one the caller talked to.
    async fn kill(&self, generation: u64) {
        let mut slot = self.host.lock().await;
        if slot.as_ref().is_some_and(|host| host.generation == generation) {
            *slot = None;
        }
    }
}

struct Host {
    generation: u64,
    stdin: ChildStdin,
    pending: Arc<Mutex<Pending>>,
    // Dropping these kills the process.
    _child: Child,
    #[cfg(windows)]
    _job: Option<crate::job::Job>,
}

impl Host {
    /// None once the host's stdout has closed.
    fn register(&self, id: u64) -> Option<oneshot::Receiver<Reply>> {
        let mut pending = self.pending.lock().unwrap();
        if !pending.open {
            return None;
        }
        let (tx, rx) = oneshot::channel();
        pending.waiters.insert(id, tx);
        Some(rx)
    }
}

struct Pending {
    open: bool,
    waiters: HashMap<u64, oneshot::Sender<Reply>>,
}

async fn read_replies(stdout: ChildStdout, pending: Arc<Mutex<Pending>>) {
    let mut lines = BufReader::new(stdout).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        match serde_json::from_str::<Reply>(&line) {
            Ok(reply) => {
                if let Some(waiter) = pending.lock().unwrap().waiters.remove(&reply.id()) {
                    let _ = waiter.send(reply);
                }
            }
            Err(e) => log::warn!(target: "zoo_module", "unreadable line from module host: {e}"),
        }
    }
    let mut pending = pending.lock().unwrap();
    pending.open = false;
    // Dropping the senders fails every call still waiting on this host.
    pending.waiters.clear();
}

async fn forward_stderr(stderr: ChildStderr) {
    let mut lines = BufReader::new(stderr).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        log::info!(target: "zoo_module", "{line}");
    }
}

fn crashed(message: impl Into<String>) -> ModuleError {
    ModuleError::Crashed { message: message.into() }
}
