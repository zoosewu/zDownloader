//! Resolution before transfer, through DownloadManager with a scripted Resolver.

mod common;

use common::{expected, Fixture};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use zoo_core::connection::Direct;
use zoo_core::resolve::{PhaseReport, Resolving};
use zoo_core::transfer::RetryPolicy;
use zoo_core::{
    AddRequest, AutoRetry, Config, CoreError, DownloadManager, Event, Phase, Resolver, Source, StartMode, Task, TaskId,
    TaskState,
};

const KB: u64 = 1024;
const SHARE_PAGE: &str = "https://share.example/f/abc";

fn config() -> Config {
    Config {
        max_concurrent: Some(3),
        connections_per_task: 2,
        segment_size: 64 * KB,
        speed_limit: None,
        auto_retry: AutoRetry { enabled: false, max_attempts: 3, interval: Duration::from_millis(50) },
        transfer_retry: RetryPolicy { max_attempts: 2, base_delay: Duration::from_millis(10), max_delay: Duration::from_millis(20) },
        user_agent: None,
    }
}

type Script = Box<dyn Fn(usize) -> Result<Source, CoreError> + Send + Sync>;

/// Answers call `n` (0-based) with `script(n)`, after reporting `Resolving`. Calls listed in
/// `hang` never finish.
struct Scripted {
    calls: AtomicUsize,
    seen: Mutex<Vec<String>>,
    hang: Vec<usize>,
    script: Script,
}

impl Scripted {
    fn new(script: impl Fn(usize) -> Result<Source, CoreError> + Send + Sync + 'static) -> Arc<Self> {
        Self::hanging(&[], script)
    }
    fn hanging(hang: &[usize], script: impl Fn(usize) -> Result<Source, CoreError> + Send + Sync + 'static) -> Arc<Self> {
        Arc::new(Self { calls: AtomicUsize::new(0), seen: Mutex::new(Vec::new()), hang: hang.to_vec(), script: Box::new(script) })
    }
    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl Resolver for Scripted {
    fn resolve<'a>(&'a self, url: &'a str, report: PhaseReport) -> Resolving<'a> {
        Box::pin(async move {
            let n = self.calls.fetch_add(1, Ordering::SeqCst);
            self.seen.lock().unwrap().push(url.to_string());
            report(Phase::Resolving { module: Some("test@local".into()) });
            if self.hang.contains(&n) {
                std::future::pending::<()>().await;
            }
            (self.script)(n)
        })
    }
}

fn expiring(url: String) -> Source {
    Source {
        url,
        headers: vec![("X-Token".into(), "t".into())],
        filename: None,
        max_connections: None,
        max_segment_size: None,
        expires: true,
    }
}

fn start(cfg: Config, store: &Path, resolver: Arc<dyn Resolver>) -> DownloadManager {
    DownloadManager::start_with(cfg, store, Arc::new(Direct::new().unwrap()), resolver).unwrap()
}

fn req(dir: &Path, filename: Option<&str>) -> AddRequest {
    AddRequest { url: SHARE_PAGE.into(), filename: filename.map(String::from), dir: dir.to_path_buf(), start: StartMode::Now, headers: Vec::new() }
}

async fn wait_until(m: &DownloadManager, id: TaskId, pred: impl Fn(&Task) -> bool) -> Task {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    loop {
        let t = m.list().await.unwrap().into_iter().find(|t| t.id == id).unwrap();
        if pred(&t) {
            return t;
        }
        assert!(tokio::time::Instant::now() < deadline, "timed out; last state: {t:#?}");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test]
async fn downloads_what_the_resolver_returns() {
    let fx = Fixture::start().await;
    let dir = tempfile::tempdir().unwrap();
    let url = fx.url("/file/200000");
    let resolver = Scripted::new(move |_| Ok(Source { filename: Some("movie.bin".into()), ..expiring(url.clone()) }));
    let m = start(config(), dir.path(), resolver.clone());
    let mut events = m.subscribe();

    let id = m.add(req(dir.path(), None)).await.unwrap().id;
    let task = wait_until(&m, id, |t| t.state == TaskState::Completed).await;

    assert_eq!(task.url, SHARE_PAGE, "the task keeps the page the user gave");
    assert_eq!(task.name, "movie.bin", "the module's name is used");
    assert_eq!(task.phase, None);
    assert_eq!(std::fs::read(task.final_path.unwrap()).unwrap(), expected(200_000));
    assert_eq!(*resolver.seen.lock().unwrap(), vec![SHARE_PAGE.to_string()]);
    assert!(fx.requests().iter().all(|r| r.headers.get("x-token").map(String::as_str) == Some("t")), "headers on every request");

    let mut phases = Vec::new();
    while let Ok(event) = events.try_recv() {
        if let Event::Upserted(t) = event {
            if let Some(p) = t.phase.filter(|p| phases.last() != Some(p)) {
                phases.push(p);
            }
        }
    }
    assert_eq!(phases, vec![Phase::Resolving { module: Some("test@local".into()) }, Phase::Transferring]);
}

#[tokio::test]
async fn the_users_file_name_wins() {
    let fx = Fixture::start().await;
    let dir = tempfile::tempdir().unwrap();
    let url = fx.url("/file/1000");
    let m = start(config(), dir.path(), Scripted::new(move |_| Ok(Source { filename: Some("module.bin".into()), ..expiring(url.clone()) })));
    let id = m.add(req(dir.path(), Some("mine.bin"))).await.unwrap().id;
    let task = wait_until(&m, id, |t| t.state == TaskState::Completed).await;
    assert_eq!(task.name, "mine.bin");
    assert!(task.final_path.unwrap().ends_with("mine.bin"));
}

#[tokio::test]
async fn an_expired_link_is_resolved_again_once() {
    let fx = Fixture::start().await;
    let dir = tempfile::tempdir().unwrap();
    let (expired, fresh) = (fx.url("/file/5000?status=403"), fx.url("/file/5000"));
    let resolver = Scripted::new(move |n| Ok(expiring(if n == 0 { expired.clone() } else { fresh.clone() })));
    let m = start(config(), dir.path(), resolver.clone());
    let id = m.add(req(dir.path(), None)).await.unwrap().id;
    wait_until(&m, id, |t| t.state == TaskState::Completed).await;
    assert_eq!(resolver.calls(), 2);

    // A link that keeps being refused fails after the second resolution.
    let always = fx.url("/file/5000?status=410");
    let resolver = Scripted::new(move |_| Ok(expiring(always.clone())));
    let dir = tempfile::tempdir().unwrap();
    let m = start(config(), dir.path(), resolver.clone());
    let id = m.add(req(dir.path(), None)).await.unwrap().id;
    let task = wait_until(&m, id, |t| t.state == TaskState::Failed).await;
    assert_eq!(task.error, Some(CoreError::Http(410)));
    assert_eq!(resolver.calls(), 2);
}

#[tokio::test]
async fn direct_links_are_not_resolved_again() {
    let fx = Fixture::start().await;
    let dir = tempfile::tempdir().unwrap();
    let url = fx.url("/file/5000?status=403");
    let resolver = Scripted::new(move |_| Ok(Source::direct(&url)));
    let m = start(config(), dir.path(), resolver.clone());
    let id = m.add(req(dir.path(), None)).await.unwrap().id;
    let task = wait_until(&m, id, |t| t.state == TaskState::Failed).await;
    assert_eq!(task.error, Some(CoreError::Http(403)));
    assert_eq!(resolver.calls(), 1);
}

#[tokio::test]
async fn resume_survives_a_new_link() {
    let fx = Fixture::start().await;
    let dir = tempfile::tempdir().unwrap();
    let size = 1024 * KB;
    // Every resolution hands out a different URL for the same file.
    let base = fx.url(&format!("/file/{size}?delay_ms=5"));
    let resolver = Scripted::new(move |n| Ok(expiring(format!("{base}&link={n}"))));
    let m = start(config(), dir.path(), resolver.clone());
    let id = m.add(req(dir.path(), None)).await.unwrap().id;

    wait_until(&m, id, |t| t.downloaded >= 128 * KB).await;
    m.pause(id).await.unwrap();
    let paused = wait_until(&m, id, |t| t.state == TaskState::Paused).await;
    let before = fx.requests().len();
    m.resume(id).await.unwrap();
    let task = wait_until(&m, id, |t| t.state == TaskState::Completed).await;

    assert_eq!(resolver.calls(), 2);
    assert_eq!(std::fs::read(task.final_path.unwrap()).unwrap(), expected(size));
    let after: Vec<_> = fx.requests()[before..].iter().filter(|r| r.range.as_deref() != Some("bytes=0-0")).cloned().collect();
    assert!(after.iter().all(|r| !r.range.as_deref().unwrap_or("bytes=0-").starts_with("bytes=0-")), "resumed, not restarted: {:?} after {} bytes", after.iter().map(|r| r.range.clone()).collect::<Vec<_>>(), paused.downloaded);
    assert!(paused.downloaded > 0);
}

#[tokio::test]
async fn resolve_failures_fail_the_task_and_honour_retryability() {
    let dir = tempfile::tempdir().unwrap();
    let fail = |retryable| move |_| Err(CoreError::Resolve { message: "file deleted".into(), hint: Some("h".into()), retryable });
    let mut cfg = config();
    cfg.auto_retry = AutoRetry { enabled: true, max_attempts: 3, interval: Duration::from_secs(3600) };

    let m = start(cfg.clone(), dir.path(), Scripted::new(fail(false)));
    let id = m.add(req(dir.path(), None)).await.unwrap().id;
    let task = wait_until(&m, id, |t| t.state == TaskState::Failed).await;
    assert!(matches!(task.error, Some(CoreError::Resolve { .. })));
    assert_eq!(task.next_retry_at, None, "not worth retrying");

    let dir = tempfile::tempdir().unwrap();
    let m = start(cfg, dir.path(), Scripted::new(fail(true)));
    let id = m.add(req(dir.path(), None)).await.unwrap().id;
    let task = wait_until(&m, id, |t| t.state == TaskState::Failed).await;
    assert!(task.next_retry_at.is_some(), "retried later");
}

#[tokio::test]
async fn pausing_during_resolution_abandons_it() {
    let fx = Fixture::start().await;
    let dir = tempfile::tempdir().unwrap();
    let url = fx.url("/file/1000");
    let resolver = Scripted::hanging(&[0], move |_| Ok(expiring(url.clone())));
    let m = start(config(), dir.path(), resolver.clone());
    let id = m.add(req(dir.path(), None)).await.unwrap().id;
    wait_until(&m, id, |t| matches!(t.phase, Some(Phase::Resolving { .. }))).await;

    let paused = m.pause(id).await.unwrap();
    assert_eq!((paused.state, paused.phase), (TaskState::Paused, None));
    m.resume(id).await.unwrap();
    wait_until(&m, id, |t| t.state == TaskState::Completed).await;
    assert_eq!(resolver.calls(), 2);
}

#[tokio::test]
async fn the_site_connection_limit_is_respected() {
    let fx = Fixture::start().await;
    let dir = tempfile::tempdir().unwrap();
    let url = fx.url(&format!("/file/{}?delay_ms=5", 1024 * KB));
    let resolver = Scripted::new(move |_| Ok(Source { max_connections: Some(1), ..expiring(url.clone()) }));
    let mut cfg = config();
    cfg.connections_per_task = 4;
    let m = start(cfg, dir.path(), resolver);
    let id = m.add(req(dir.path(), None)).await.unwrap().id;

    let mut most = 0;
    loop {
        let task = m.list().await.unwrap().into_iter().find(|t| t.id == id).unwrap();
        if task.state == TaskState::Completed {
            break;
        }
        most = most.max(m.segments(id).await.unwrap().active.len());
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(most, 1, "one connection at a time");
}

#[tokio::test]
async fn the_site_segment_limit_is_respected() {
    let fx = Fixture::start().await;
    let dir = tempfile::tempdir().unwrap();
    let size = 300 * KB;
    let url = fx.url(&format!("/file/{size}"));
    // Larger configured segments (64 KiB) than the site allows (16 KiB).
    let resolver = Scripted::new(move |_| Ok(Source { max_segment_size: Some(16 * KB), ..expiring(url.clone()) }));
    let m = start(config(), dir.path(), resolver);
    let id = m.add(req(dir.path(), None)).await.unwrap().id;
    let task = wait_until(&m, id, |t| t.state == TaskState::Completed).await;
    assert_eq!(std::fs::read(task.final_path.unwrap()).unwrap(), expected(size));

    let ranges: Vec<(u64, u64)> = fx
        .requests()
        .iter()
        .filter_map(|r| r.range.as_deref()?.strip_prefix("bytes=")?.split_once('-').map(|(a, b)| (a.parse().unwrap(), b.parse().unwrap())))
        .filter(|&(start, end)| (start, end) != (0, 0)) // the probe
        .collect();
    assert!(ranges.len() >= (size / (16 * KB)) as usize, "{} ranges", ranges.len());
    assert!(ranges.iter().all(|&(start, end)| end - start < 16 * KB), "a range exceeded 16 KiB: {ranges:?}");
}

#[tokio::test]
async fn resolved_headers_can_replace_the_user_agent() {
    let fx = Fixture::start().await;
    let dir = tempfile::tempdir().unwrap();
    let url = fx.url("/file/1000");
    let headers = vec![("User-Agent".to_string(), "netdisk;7.42.0.5;PC".to_string())];
    let cfg = Config { user_agent: Some("configured".into()), ..config() };
    let m = start(cfg, dir.path(), Scripted::new(move |_| Ok(Source { headers: headers.clone(), ..expiring(url.clone()) })));
    // The task's own headers apply too, unless the module sets the same one.
    let own = vec![("user-agent".to_string(), "task".to_string()), ("Referer".to_string(), "https://share.example/".to_string())];
    let id = m.add(AddRequest { headers: own, ..req(dir.path(), None) }).await.unwrap().id;
    wait_until(&m, id, |t| t.state == TaskState::Completed).await;
    let requests = fx.requests();
    let agents: Vec<_> = requests.iter().map(|r| r.headers.get("user-agent").cloned()).collect();
    assert!(agents.iter().all(|ua| ua.as_deref() == Some("netdisk;7.42.0.5;PC")), "{agents:?}");
    assert!(requests.iter().all(|r| r.headers.get("referer").map(String::as_str) == Some("https://share.example/")));
}
