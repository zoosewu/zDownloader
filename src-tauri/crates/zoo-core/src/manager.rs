//! DownloadManager: the single owner of every task.
//!
//! One actor task holds all state; the handle only sends it commands. Each running task has
//! a transfer job with its own cancellation token. A job that is stopping stays in `jobs`
//! until it reports back, so a task is never transferred by two jobs at once even if the
//! user pauses and resumes quickly.
//!
//! Commands change the visible state immediately (pause shows Paused at once); the job's
//! final report only fills in what the command could not know (completion, failure).

use crate::connection::ConnectionSource;
use crate::error::CoreError;
use crate::naming;
use crate::part_file::PartFile;
use crate::rate::RateLimiter;
use crate::store::JsonStore;
use crate::task::{AddRequest, Config, Event, ProgressTick, StartMode, Task, TaskId, TaskState};
use crate::transfer::{self, Ctx, Progress, TransferSpec};
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_util::sync::CancellationToken;

const TICK: Duration = Duration::from_millis(250);
const EVENT_BUFFER: usize = 1024;

#[derive(Clone)]
pub struct DownloadManager {
    tx: mpsc::UnboundedSender<Cmd>,
    events: broadcast::Sender<Event>,
}

type Reply<T> = oneshot::Sender<Result<T, CoreError>>;

enum Cmd {
    Add(AddRequest, Reply<Task>),
    Pause(TaskId, Reply<Task>),
    Resume(TaskId, Reply<Task>),
    Cancel(TaskId, Reply<Task>),
    Retry(TaskId, Reply<Task>),
    Remove(TaskId, Reply<()>),
    PauseAll(oneshot::Sender<Vec<Task>>),
    ResumeAll(oneshot::Sender<Vec<Task>>),
    ClearFinished(oneshot::Sender<Vec<TaskId>>),
    List(oneshot::Sender<Vec<Task>>),
    UpdateConfig(Config),
    SetConnection(Arc<dyn ConnectionSource>),
    Shutdown(oneshot::Sender<()>),
}

impl DownloadManager {
    /// Load tasks from `store_dir/tasks.json` and start the actor on the current Tokio runtime.
    /// Tasks that were queued or running when the app last stopped are queued again.
    pub fn start(cfg: Config, store_dir: &Path, conn: Arc<dyn ConnectionSource>) -> Result<Self, CoreError> {
        let store = JsonStore::new(store_dir.join("tasks.json"));
        let mut tasks: Vec<Task> = store.load()?.unwrap_or_default();
        let rev = tasks.iter().map(|t| t.rev).max().unwrap_or(0);
        for task in &mut tasks {
            task.speed = 0;
            if matches!(task.state, TaskState::Running | TaskState::Queued) {
                task.state = TaskState::Queued;
            }
        }

        let (tx, cmd_rx) = mpsc::unbounded_channel();
        let (done_tx, done_rx) = mpsc::unbounded_channel();
        let (events, _) = broadcast::channel(EVENT_BUFFER);
        let actor = Actor {
            rate: Arc::new(RateLimiter::new(cfg.speed_limit)),
            cfg,
            conn,
            tasks,
            jobs: HashMap::new(),
            store,
            events: events.clone(),
            done_tx,
            shutting_down: false,
            rev,
        };
        tokio::spawn(actor.run(cmd_rx, done_rx));
        Ok(Self { tx, events })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.events.subscribe()
    }

    pub async fn add(&self, req: AddRequest) -> Result<Task, CoreError> {
        self.ask(|r| Cmd::Add(req, r)).await?
    }
    pub async fn pause(&self, id: TaskId) -> Result<Task, CoreError> {
        self.ask(|r| Cmd::Pause(id, r)).await?
    }
    pub async fn resume(&self, id: TaskId) -> Result<Task, CoreError> {
        self.ask(|r| Cmd::Resume(id, r)).await?
    }
    /// Stop and delete partial data.
    pub async fn cancel(&self, id: TaskId) -> Result<Task, CoreError> {
        self.ask(|r| Cmd::Cancel(id, r)).await?
    }
    /// Start a failed or cancelled task again, resetting its retry count.
    pub async fn retry(&self, id: TaskId) -> Result<Task, CoreError> {
        self.ask(|r| Cmd::Retry(id, r)).await?
    }
    /// Drop the task from the list. Partial data is deleted; a finished file is kept.
    pub async fn remove(&self, id: TaskId) -> Result<(), CoreError> {
        self.ask(|r| Cmd::Remove(id, r)).await?
    }
    pub async fn pause_all(&self) -> Result<Vec<Task>, CoreError> {
        self.ask(Cmd::PauseAll).await
    }
    pub async fn resume_all(&self) -> Result<Vec<Task>, CoreError> {
        self.ask(Cmd::ResumeAll).await
    }
    /// Remove completed and cancelled tasks from the list.
    pub async fn clear_finished(&self) -> Result<Vec<TaskId>, CoreError> {
        self.ask(Cmd::ClearFinished).await
    }
    pub async fn list(&self) -> Result<Vec<Task>, CoreError> {
        self.ask(Cmd::List).await
    }
    pub fn update_config(&self, cfg: Config) {
        let _ = self.tx.send(Cmd::UpdateConfig(cfg));
    }
    pub fn set_connection_source(&self, conn: Arc<dyn ConnectionSource>) {
        let _ = self.tx.send(Cmd::SetConnection(conn));
    }
    /// Stop all transfers, keeping their progress and state for the next start.
    pub async fn shutdown(&self) {
        let _ = self.ask(Cmd::Shutdown).await;
    }

    async fn ask<T>(&self, make: impl FnOnce(oneshot::Sender<T>) -> Cmd) -> Result<T, CoreError> {
        let (reply, rx) = oneshot::channel();
        self.tx.send(make(reply)).map_err(|_| stopped())?;
        rx.await.map_err(|_| stopped())
    }
}

fn stopped() -> CoreError {
    CoreError::Invalid("download manager has stopped".into())
}

/// Why a job was told to stop; decides what its cancellation means.
#[derive(Clone, Copy, PartialEq)]
enum Stop {
    Pause,
    Cancel,
    Remove,
    Shutdown,
}

struct Job {
    cancel: CancellationToken,
    progress: Arc<Progress>,
    part_path: PathBuf,
    stop: Option<Stop>,
    speed: SpeedMeter,
}

struct Actor {
    cfg: Config,
    conn: Arc<dyn ConnectionSource>,
    rate: Arc<RateLimiter>,
    /// In insertion order, which is also the queue order.
    tasks: Vec<Task>,
    jobs: HashMap<TaskId, Job>,
    store: JsonStore<Vec<Task>>,
    events: broadcast::Sender<Event>,
    done_tx: mpsc::UnboundedSender<(TaskId, Result<PathBuf, CoreError>)>,
    shutting_down: bool,
    /// Last revision handed out; see `Task::rev`.
    rev: u64,
}

impl Actor {
    async fn run(
        mut self,
        mut cmd_rx: mpsc::UnboundedReceiver<Cmd>,
        mut done_rx: mpsc::UnboundedReceiver<(TaskId, Result<PathBuf, CoreError>)>,
    ) {
        self.wake_due();
        self.pump();
        self.save();
        let mut tick = tokio::time::interval(TICK);
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            let wake = self.next_wakeup();
            tokio::select! {
                cmd = cmd_rx.recv() => match cmd {
                    Some(Cmd::Shutdown(reply)) => {
                        self.shutdown(&mut done_rx).await;
                        let _ = reply.send(());
                        return;
                    }
                    Some(cmd) => self.handle(cmd),
                    // Every handle dropped: stop like a shutdown.
                    None => return self.shutdown(&mut done_rx).await,
                },
                Some((id, result)) = done_rx.recv() => self.finished(id, result),
                _ = tick.tick() => self.progress_tick(),
                _ = sleep_for(wake) => self.wake_due(),
            }
        }
    }

    fn handle(&mut self, cmd: Cmd) {
        match cmd {
            Cmd::Add(req, reply) => {
                let _ = reply.send(self.add(req));
            }
            Cmd::Pause(id, reply) => {
                let _ = reply.send(self.pause(id));
            }
            Cmd::Resume(id, reply) => {
                let _ = reply.send(self.resume(id));
            }
            Cmd::Cancel(id, reply) => {
                let _ = reply.send(self.cancel(id));
            }
            Cmd::Retry(id, reply) => {
                let _ = reply.send(self.retry(id));
            }
            Cmd::Remove(id, reply) => {
                let _ = reply.send(self.remove(id));
            }
            Cmd::PauseAll(reply) => {
                let ids = self.ids_where(|t| matches!(t.state, TaskState::Running | TaskState::Queued | TaskState::Scheduled));
                let _ = reply.send(ids.into_iter().filter_map(|id| self.pause(id).ok()).collect());
            }
            Cmd::ResumeAll(reply) => {
                let ids = self.ids_where(|t| matches!(t.state, TaskState::Paused | TaskState::Failed));
                let _ = reply.send(ids.into_iter().filter_map(|id| self.resume(id).ok()).collect());
            }
            Cmd::ClearFinished(reply) => {
                let ids = self.ids_where(|t| t.state.is_finished());
                for id in &ids {
                    let _ = self.remove(*id);
                }
                let _ = reply.send(ids);
            }
            Cmd::List(reply) => {
                let _ = reply.send(self.tasks.clone());
            }
            Cmd::UpdateConfig(cfg) => {
                self.rate.set_rate(cfg.speed_limit);
                self.cfg = cfg;
                self.pump();
            }
            Cmd::SetConnection(conn) => self.conn = conn,
            Cmd::Shutdown(_) => unreachable!("handled in run"),
        }
        self.save();
    }

    fn add(&mut self, req: AddRequest) -> Result<Task, CoreError> {
        let parsed = url::Url::parse(&req.url).map_err(|e| CoreError::Invalid(format!("invalid URL: {e}")))?;
        if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
            return Err(CoreError::Invalid("URL must be http(s) with a host".into()));
        }
        let (state, scheduled_at) = match req.start {
            StartMode::Now => (TaskState::Queued, None),
            StartMode::Later => (TaskState::Paused, None),
            StartMode::At(at) => (TaskState::Scheduled, Some(at)),
        };
        self.rev += 1;
        let task = Task {
            id: TaskId::new(),
            name: naming::filename_for(req.filename.as_deref(), &req.url),
            url: req.url,
            dir: req.dir,
            final_path: None,
            state,
            error: None,
            downloaded: 0,
            total: None,
            speed: 0,
            created_at: Utc::now(),
            started_at: None,
            finished_at: None,
            scheduled_at,
            retry_count: 0,
            next_retry_at: None,
            rev: self.rev,
        };
        self.tasks.push(task.clone());
        self.emit(Event::Upserted(task.clone()));
        self.pump();
        self.wake_due();
        self.snapshot(task.id)
    }

    fn pause(&mut self, id: TaskId) -> Result<Task, CoreError> {
        let state = self.task(id)?.state;
        match state {
            TaskState::Running => self.stop_job(id, Stop::Pause),
            TaskState::Queued | TaskState::Scheduled | TaskState::Failed => {}
            TaskState::Paused => return self.snapshot(id),
            s @ (TaskState::Completed | TaskState::Cancelled) => return Err(invalid_transition(s, "pause")),
        }
        self.set(id, |t| {
            t.state = TaskState::Paused;
            t.next_retry_at = None;
            t.speed = 0;
        })
    }

    fn resume(&mut self, id: TaskId) -> Result<Task, CoreError> {
        let state = self.task(id)?.state;
        match state {
            TaskState::Paused | TaskState::Scheduled => self.set(id, |t| t.state = TaskState::Queued)?,
            TaskState::Failed => return self.retry(id),
            TaskState::Queued | TaskState::Running => return self.snapshot(id),
            s @ (TaskState::Completed | TaskState::Cancelled) => return Err(invalid_transition(s, "resume")),
        };
        self.pump();
        self.snapshot(id)
    }

    fn retry(&mut self, id: TaskId) -> Result<Task, CoreError> {
        let state = self.task(id)?.state;
        match state {
            TaskState::Failed | TaskState::Cancelled => {}
            s => return Err(invalid_transition(s, "retry")),
        }
        self.set(id, |t| {
            t.state = TaskState::Queued;
            t.error = None;
            t.retry_count = 0;
            t.next_retry_at = None;
            t.finished_at = None;
        })?;
        self.pump();
        self.snapshot(id)
    }

    fn cancel(&mut self, id: TaskId) -> Result<Task, CoreError> {
        let (state, part) = self.task(id).map(|t| (t.state, part_path_of(t)))?;
        match state {
            TaskState::Running => self.stop_job(id, Stop::Cancel),
            TaskState::Completed | TaskState::Cancelled => return Err(invalid_transition(state, "cancel")),
            // A previous job may still be stopping (e.g. paused then cancelled at once).
            _ if self.jobs.contains_key(&id) => self.stop_job(id, Stop::Cancel),
            _ => discard_part(&part),
        }
        self.set(id, |t| {
            t.state = TaskState::Cancelled;
            t.downloaded = 0;
            t.speed = 0;
            t.next_retry_at = None;
            t.finished_at = Some(Utc::now());
        })
    }

    fn remove(&mut self, id: TaskId) -> Result<(), CoreError> {
        let (state, part) = self.task(id).map(|t| (t.state, part_path_of(t)))?;
        match state {
            TaskState::Completed => {}
            // Running, or a previous job still stopping: its report deletes the part.
            _ if self.jobs.contains_key(&id) => self.stop_job(id, Stop::Remove),
            _ => discard_part(&part),
        }
        self.tasks.retain(|t| t.id != id);
        self.emit(Event::Removed(id));
        Ok(())
    }

    fn stop_job(&mut self, id: TaskId, stop: Stop) {
        if let Some(job) = self.jobs.get_mut(&id) {
            job.stop = Some(stop);
            job.cancel.cancel();
        }
    }

    /// Start queued tasks while there are free slots.
    fn pump(&mut self) {
        if self.shutting_down {
            return;
        }
        loop {
            if self.cfg.max_concurrent.is_some_and(|max| self.jobs.len() >= max) {
                return;
            }
            let next = self
                .tasks
                .iter()
                .find(|t| t.state == TaskState::Queued && !self.jobs.contains_key(&t.id))
                .map(|t| t.id);
            let Some(id) = next else { return };
            self.start_job(id);
        }
    }

    fn start_job(&mut self, id: TaskId) {
        let task = self.tasks.iter().find(|t| t.id == id).expect("queued task exists");
        let spec = TransferSpec {
            url: task.url.clone(),
            dir: task.dir.clone(),
            name: task.name.clone(),
            part_path: part_path_of(task),
            connections: self.cfg.connections_per_task,
            segment_size: self.cfg.segment_size,
            retry: self.cfg.transfer_retry,
        };
        let ctx = Ctx {
            conn: self.conn.clone(),
            rate: self.rate.clone(),
            progress: Arc::new(Progress::default()),
            cancel: CancellationToken::new(),
        };
        self.jobs.insert(
            id,
            Job {
                cancel: ctx.cancel.clone(),
                progress: ctx.progress.clone(),
                part_path: spec.part_path.clone(),
                stop: None,
                speed: SpeedMeter::default(),
            },
        );
        let done = self.done_tx.clone();
        tokio::spawn(async move {
            let result = transfer::run(&spec, &ctx).await;
            let _ = done.send((id, result));
        });
        let _ = self.set(id, |t| {
            t.state = TaskState::Running;
            t.error = None;
            t.started_at.get_or_insert_with(Utc::now);
        });
    }

    fn finished(&mut self, id: TaskId, result: Result<PathBuf, CoreError>) {
        let Some(job) = self.jobs.remove(&id) else { return };
        let still_listed = self.tasks.iter().any(|t| t.id == id);

        match (&result, job.stop) {
            (Ok(_), _) if still_listed => {}
            // Removed while running: nothing left to update, but its partial data must go.
            (_, Some(Stop::Remove)) | (Err(_), Some(Stop::Cancel)) => discard_part(&job.part_path),
            _ => {}
        }

        if still_listed {
            let downloaded = job.progress.downloaded();
            let total = job.progress.total();
            match result {
                Ok(path) => {
                    let _ = self.set(id, |t| {
                        t.state = TaskState::Completed;
                        t.final_path = Some(path);
                        t.total = total.or(Some(downloaded));
                        t.downloaded = downloaded;
                        t.speed = 0;
                        t.error = None;
                        t.finished_at = Some(Utc::now());
                    });
                }
                // The stop command already set the visible state.
                Err(CoreError::Cancelled) => {
                    if job.stop != Some(Stop::Cancel) {
                        let _ = self.set(id, |t| {
                            t.downloaded = downloaded;
                            t.total = total.or(t.total);
                        });
                    }
                }
                Err(e) => self.fail(id, e, downloaded, total),
            }
        }
        self.save();
        self.pump();
    }

    fn fail(&mut self, id: TaskId, error: CoreError, downloaded: u64, total: Option<u64>) {
        let auto = self.cfg.auto_retry;
        let _ = self.set(id, |t| {
            t.state = TaskState::Failed;
            t.downloaded = downloaded;
            t.total = total.or(t.total);
            t.speed = 0;
            t.finished_at = Some(Utc::now());
            t.next_retry_at = None;
            if auto.enabled && error.is_retryable() && t.retry_count < auto.max_attempts {
                t.retry_count += 1;
                let factor = 1u32.checked_shl(t.retry_count - 1).unwrap_or(u32::MAX);
                let delay = auto.interval.saturating_mul(factor);
                t.next_retry_at = chrono::Duration::from_std(delay).ok().map(|d| Utc::now() + d);
            }
            t.error = Some(error);
        });
    }

    /// Move scheduled and retry-pending tasks whose time has come into the queue.
    fn wake_due(&mut self) {
        let now = Utc::now();
        let due = self.ids_where(|t| match t.state {
            TaskState::Scheduled => t.scheduled_at.is_none_or(|at| at <= now),
            TaskState::Failed => t.next_retry_at.is_some_and(|at| at <= now),
            _ => false,
        });
        for id in due {
            let _ = self.set(id, |t| {
                t.state = TaskState::Queued;
                t.next_retry_at = None;
            });
        }
        self.pump();
    }

    fn next_wakeup(&self) -> Option<Duration> {
        let now = Utc::now();
        self.tasks
            .iter()
            .filter_map(|t| match t.state {
                TaskState::Scheduled => t.scheduled_at,
                TaskState::Failed => t.next_retry_at,
                _ => None,
            })
            .min()
            .map(|at| (at - now).to_std().unwrap_or(Duration::ZERO))
    }

    fn progress_tick(&mut self) {
        let mut ticks = Vec::new();
        for (id, job) in &mut self.jobs {
            if job.stop.is_some() {
                continue;
            }
            let downloaded = job.progress.downloaded();
            let total = job.progress.total();
            let speed = job.speed.sample(downloaded);
            if let Some(task) = self.tasks.iter_mut().find(|t| t.id == *id) {
                task.downloaded = downloaded;
                task.total = total;
                task.speed = speed;
                ticks.push(ProgressTick { id: *id, downloaded, total, speed });
            }
        }
        if !ticks.is_empty() {
            self.emit(Event::Progress(ticks));
        }
    }

    async fn shutdown(&mut self, done_rx: &mut mpsc::UnboundedReceiver<(TaskId, Result<PathBuf, CoreError>)>) {
        self.shutting_down = true;
        for job in self.jobs.values_mut() {
            job.stop.get_or_insert(Stop::Shutdown);
            job.cancel.cancel();
        }
        while !self.jobs.is_empty() {
            let Some((id, result)) = done_rx.recv().await else { break };
            self.finished_on_shutdown(id, result);
        }
        self.save();
    }

    /// Like `finished`, but a cancelled transfer keeps its Running state so the next start resumes it.
    fn finished_on_shutdown(&mut self, id: TaskId, result: Result<PathBuf, CoreError>) {
        match &result {
            Err(CoreError::Cancelled) if self.jobs.get(&id).is_some_and(|j| j.stop == Some(Stop::Shutdown)) => {
                let job = self.jobs.remove(&id).unwrap();
                let (downloaded, total) = (job.progress.downloaded(), job.progress.total());
                let _ = self.set(id, |t| {
                    t.downloaded = downloaded;
                    t.total = total.or(t.total);
                    t.speed = 0;
                });
            }
            _ => self.finished(id, result),
        }
    }

    // --- helpers -------------------------------------------------------------------------

    fn task(&self, id: TaskId) -> Result<&Task, CoreError> {
        self.tasks.iter().find(|t| t.id == id).ok_or(CoreError::NotFound)
    }

    fn snapshot(&self, id: TaskId) -> Result<Task, CoreError> {
        self.task(id).cloned()
    }

    fn ids_where(&self, pred: impl Fn(&Task) -> bool) -> Vec<TaskId> {
        self.tasks.iter().filter(|t| pred(t)).map(|t| t.id).collect()
    }

    /// Mutate a task and announce the change.
    fn set(&mut self, id: TaskId, f: impl FnOnce(&mut Task)) -> Result<Task, CoreError> {
        let task = self.tasks.iter_mut().find(|t| t.id == id).ok_or(CoreError::NotFound)?;
        f(task);
        self.rev += 1;
        task.rev = self.rev;
        let task = task.clone();
        self.emit(Event::Upserted(task.clone()));
        Ok(task)
    }

    fn emit(&self, event: Event) {
        // No subscribers is fine.
        let _ = self.events.send(event);
    }

    fn save(&self) {
        if let Err(e) = self.store.save(&self.tasks) {
            log::error!("failed to save {}: {e}", self.store.path().display());
        }
    }
}

fn part_path_of(task: &Task) -> PathBuf {
    naming::part_path(&task.dir, &task.name, &task.id.to_string())
}

fn discard_part(path: &Path) {
    if let Err(e) = PartFile::discard(path) {
        log::warn!("failed to delete {}: {e}", path.display());
    }
}

fn invalid_transition(state: TaskState, action: &str) -> CoreError {
    CoreError::Invalid(format!("cannot {action} a {state:?} task"))
}

async fn sleep_for(wait: Option<Duration>) {
    match wait {
        Some(d) => tokio::time::sleep(d).await,
        None => std::future::pending().await,
    }
}

/// Exponentially smoothed bytes-per-second.
#[derive(Default)]
struct SpeedMeter {
    last: Option<(Instant, u64)>,
    rate: f64,
}

impl SpeedMeter {
    fn sample(&mut self, downloaded: u64) -> u64 {
        let now = Instant::now();
        if let Some((at, bytes)) = self.last {
            let secs = now.duration_since(at).as_secs_f64();
            if secs > 0.0 {
                let instant = downloaded.saturating_sub(bytes) as f64 / secs;
                self.rate = if self.rate == 0.0 { instant } else { 0.3 * instant + 0.7 * self.rate };
            }
        }
        self.last = Some((now, downloaded));
        self.rate.round() as u64
    }
}
