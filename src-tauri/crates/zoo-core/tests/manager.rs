//! DownloadManager behaviour, exercised only through its public interface.

mod common;

use chrono::Utc;
use common::{expected, Fixture};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use zoo_core::connection::Direct;
use zoo_core::transfer::RetryPolicy;
use zoo_core::{AddRequest, AutoRetry, Config, CoreError, DownloadManager, Event, StartMode, Task, TaskId, TaskState};

const KB: u64 = 1024;

fn config() -> Config {
    Config {
        max_concurrent: Some(3),
        connections_per_task: 2,
        segment_size: 64 * KB,
        speed_limit: None,
        auto_retry: AutoRetry { enabled: false, max_attempts: 3, interval: Duration::from_millis(50) },
        transfer_retry: RetryPolicy { max_attempts: 2, base_delay: Duration::from_millis(10), max_delay: Duration::from_millis(20) },
    }
}

fn start(cfg: Config, store: &Path) -> DownloadManager {
    DownloadManager::start(cfg, store, Arc::new(Direct::new().unwrap())).unwrap()
}

fn req(url: String, dir: &Path, start: StartMode) -> AddRequest {
    AddRequest { url, filename: None, dir: dir.to_path_buf(), start }
}

async fn get(m: &DownloadManager, id: TaskId) -> Task {
    m.list().await.unwrap().into_iter().find(|t| t.id == id).expect("task listed")
}

/// Poll until `pred` holds for the task, or panic after `secs`.
async fn wait_until(m: &DownloadManager, id: TaskId, secs: u64, pred: impl Fn(&Task) -> bool) -> Task {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(secs);
    loop {
        let t = get(m, id).await;
        if pred(&t) {
            return t;
        }
        assert!(tokio::time::Instant::now() < deadline, "timed out; last state: {t:#?}");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

async fn wait_state(m: &DownloadManager, id: TaskId, state: TaskState) -> Task {
    wait_until(m, id, 15, |t| t.state == state).await
}

fn assert_downloaded(task: &Task, size: u64) {
    let path = task.final_path.as_ref().expect("final path");
    assert!(std::fs::read(path).unwrap() == expected(size), "content mismatch in {}", path.display());
}

fn part_files(dir: &Path) -> usize {
    std::fs::read_dir(dir).unwrap().filter(|e| e.as_ref().unwrap().file_name().to_string_lossy().ends_with(".part")).count()
}

#[tokio::test]
async fn added_task_downloads_and_announces_each_state() {
    let fx = Fixture::start().await;
    let (dir, store) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let m = start(config(), store.path());
    let mut events = m.subscribe();

    let task = m.add(req(fx.url("/file/300000"), dir.path(), StartMode::Now)).await.unwrap();
    assert_eq!(task.name, "300000");
    let done = wait_state(&m, task.id, TaskState::Completed).await;

    assert_downloaded(&done, 300_000);
    assert_eq!((done.downloaded, done.total), (300_000, Some(300_000)));
    let mut states = Vec::new();
    while let Ok(ev) = events.try_recv() {
        if let Event::Upserted(t) = ev {
            if states.last() != Some(&t.state) {
                states.push(t.state);
            }
        }
    }
    assert_eq!(states, vec![TaskState::Queued, TaskState::Running, TaskState::Completed]);
}

#[tokio::test]
async fn concurrency_limit_is_enforced_and_follows_config_changes() {
    let fx = Fixture::start().await;
    let (dir, store) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let m = start(Config { max_concurrent: Some(2), ..config() }, store.path());

    let mut ids = Vec::new();
    for i in 0..4 {
        let url = fx.url(&format!("/file/{}?delay_ms=20&n={i}", 1024 * KB));
        ids.push(m.add(req(url, dir.path(), StartMode::Now)).await.unwrap().id);
    }
    tokio::time::sleep(Duration::from_millis(200)).await;
    let running = |tasks: &[Task]| tasks.iter().filter(|t| t.state == TaskState::Running).count();
    assert_eq!(running(&m.list().await.unwrap()), 2);

    m.update_config(Config { max_concurrent: Some(4), ..config() });
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(running(&m.list().await.unwrap()), 4);
    for id in ids {
        wait_state(&m, id, TaskState::Completed).await;
    }
}

#[tokio::test]
async fn pause_and_resume_continue_from_saved_progress() {
    let fx = Fixture::start().await;
    let (dir, store) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let m = start(config(), store.path());
    let size = 1024 * KB;

    let id = m.add(req(fx.url(&format!("/file/{size}?delay_ms=5")), dir.path(), StartMode::Now)).await.unwrap().id;
    wait_until(&m, id, 15, |t| t.downloaded > 200 * KB).await;
    let paused = m.pause(id).await.unwrap();
    assert_eq!(paused.state, TaskState::Paused);
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(get(&m, id).await.state, TaskState::Paused, "a paused task must not restart by itself");
    assert_eq!(part_files(dir.path()), 1, "pause keeps the part file");

    let before = fx.requests().len();
    m.resume(id).await.unwrap();
    let done = wait_state(&m, id, TaskState::Completed).await;
    assert_downloaded(&done, size);
    let refetched: u64 = fx.requests()[before..]
        .iter()
        .filter_map(|r| r.range.as_deref())
        .filter(|v| *v != "bytes=0-0")
        .map(|v| {
            let (s, e) = v.trim_start_matches("bytes=").split_once('-').unwrap();
            e.parse::<u64>().unwrap() - s.parse::<u64>().unwrap() + 1
        })
        .sum();
    assert!(refetched < size, "resume refetched everything ({refetched} bytes)");
}

#[tokio::test]
async fn cancel_deletes_partial_data_and_retry_restarts() {
    let fx = Fixture::start().await;
    let (dir, store) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let m = start(config(), store.path());
    let size = 1024 * KB;

    let id = m.add(req(fx.url(&format!("/file/{size}?delay_ms=5")), dir.path(), StartMode::Now)).await.unwrap().id;
    wait_until(&m, id, 15, |t| t.downloaded > 100 * KB).await;
    assert_eq!(m.cancel(id).await.unwrap().state, TaskState::Cancelled);
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(part_files(dir.path()), 0);
    assert!(matches!(m.pause(id).await, Err(CoreError::Invalid(_))));

    m.retry(id).await.unwrap();
    assert_downloaded(&wait_state(&m, id, TaskState::Completed).await, size);
}

#[tokio::test]
async fn remove_running_task_cleans_up() {
    let fx = Fixture::start().await;
    let (dir, store) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let m = start(config(), store.path());
    let mut events = m.subscribe();

    let id = m.add(req(fx.url(&format!("/file/{}?delay_ms=5", 1024 * KB)), dir.path(), StartMode::Now)).await.unwrap().id;
    wait_until(&m, id, 15, |t| t.downloaded > 0).await;
    m.remove(id).await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    assert!(m.list().await.unwrap().is_empty());
    assert_eq!(part_files(dir.path()), 0);
    let mut removed = false;
    while let Ok(ev) = events.try_recv() {
        removed |= ev == Event::Removed(id);
    }
    assert!(removed);
}

#[tokio::test]
async fn interrupted_downloads_resume_automatically_after_restart() {
    let fx = Fixture::start().await;
    let (dir, store) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let size = 2048 * KB;
    let url = fx.url(&format!("/file/{size}?delay_ms=5"));

    let first = start(config(), store.path());
    let id = first.add(req(url, dir.path(), StartMode::Now)).await.unwrap().id;
    let waiting = first.add(req(fx.url("/file/1000"), dir.path(), StartMode::Later)).await.unwrap().id;
    wait_until(&first, id, 15, |t| t.downloaded > 300 * KB).await;
    first.shutdown().await;
    assert!(first.list().await.is_err(), "manager is stopped");

    let second = start(config(), store.path());
    let resumed = get(&second, id).await;
    assert!(matches!(resumed.state, TaskState::Queued | TaskState::Running), "{:?}", resumed.state);
    assert!(resumed.downloaded > 0, "progress survives restart");
    assert_eq!(get(&second, waiting).await.state, TaskState::Paused, "paused tasks stay paused");
    assert_downloaded(&wait_state(&second, id, TaskState::Completed).await, size);
}

#[tokio::test]
async fn transient_failures_retry_automatically() {
    let fx = Fixture::start().await;
    let (dir, store) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let cfg = Config { auto_retry: AutoRetry { enabled: true, ..config().auto_retry }, ..config() };
    let m = start(cfg, store.path());

    // Each transfer tries twice; four 503s exhaust the first transfer and part of the next.
    let id = m.add(req(fx.url("/file/5000?fail=503&times=3"), dir.path(), StartMode::Now)).await.unwrap().id;
    let done = wait_state(&m, id, TaskState::Completed).await;

    assert_eq!(done.retry_count, 1);
    assert_eq!(done.error, None);
    assert_downloaded(&done, 5000);
}

#[tokio::test]
async fn permanent_failures_are_not_retried_until_the_user_asks() {
    let fx = Fixture::start().await;
    let (dir, store) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let cfg = Config { auto_retry: AutoRetry { enabled: true, ..config().auto_retry }, ..config() };
    let m = start(cfg, store.path());

    let id = m.add(req(fx.url("/file/10?status=404"), dir.path(), StartMode::Now)).await.unwrap().id;
    let failed = wait_state(&m, id, TaskState::Failed).await;
    assert_eq!(failed.error, Some(CoreError::Http(404)));
    assert_eq!((failed.retry_count, failed.next_retry_at), (0, None));
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(get(&m, id).await.state, TaskState::Failed);
}

#[tokio::test]
async fn scheduled_task_waits_for_its_time() {
    let fx = Fixture::start().await;
    let (dir, store) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let m = start(config(), store.path());
    let at = Utc::now() + chrono::Duration::milliseconds(400);

    let id = m.add(req(fx.url("/file/1000"), dir.path(), StartMode::At(at))).await.unwrap().id;
    tokio::time::sleep(Duration::from_millis(200)).await;
    let (task, now) = (get(&m, id).await, Utc::now());
    // Only meaningful if the machine was not so busy that the sleep overshot `at`.
    if now < at {
        assert_eq!(task.state, TaskState::Scheduled);
        assert!(fx.requests().is_empty());
    }

    let done = wait_state(&m, id, TaskState::Completed).await;
    assert!(done.started_at.unwrap() >= at, "started before its scheduled time");
}

#[tokio::test]
async fn added_later_waits_for_resume_and_bulk_controls_work() {
    let fx = Fixture::start().await;
    let (dir, store) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let m = start(config(), store.path());

    let a = m.add(req(fx.url("/file/1000?n=a"), dir.path(), StartMode::Later)).await.unwrap().id;
    let b = m.add(req(fx.url("/file/1000?n=b"), dir.path(), StartMode::Later)).await.unwrap().id;
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(fx.requests().is_empty());

    assert_eq!(m.resume_all().await.unwrap().len(), 2);
    wait_state(&m, a, TaskState::Completed).await;
    wait_state(&m, b, TaskState::Completed).await;

    let cleared = m.clear_finished().await.unwrap();
    assert_eq!(cleared.len(), 2);
    assert!(m.list().await.unwrap().is_empty());
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2, "clearing keeps finished files");
}

#[tokio::test]
async fn add_rejects_bad_urls_and_sanitizes_names() {
    let fx = Fixture::start().await;
    let (dir, store) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let m = start(config(), store.path());

    for bad in ["file:///etc/passwd", "javascript:alert(1)", "not a url"] {
        assert!(matches!(m.add(req(bad.into(), dir.path(), StartMode::Now)).await, Err(CoreError::Invalid(_))), "{bad}");
    }
    let mut r = req(fx.url("/file/10"), dir.path(), StartMode::Later);
    r.filename = Some(r"..\..\Startup\evil.bat".into());
    assert_eq!(m.add(r).await.unwrap().name, "evil.bat");
}

#[tokio::test]
async fn revisions_increase_with_every_change_and_survive_restart() {
    let fx = Fixture::start().await;
    let (dir, store) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let m = start(config(), store.path());
    let mut events = m.subscribe();

    let id = m.add(req(fx.url("/file/1000"), dir.path(), StartMode::Now)).await.unwrap().id;
    let done = wait_state(&m, id, TaskState::Completed).await;
    let mut revs = Vec::new();
    while let Ok(ev) = events.try_recv() {
        if let Event::Upserted(t) = ev {
            revs.push(t.rev);
        }
    }
    assert!(revs.windows(2).all(|w| w[0] < w[1]), "not strictly increasing: {revs:?}");
    assert_eq!(*revs.last().unwrap(), done.rev);
    m.shutdown().await;

    let again = start(config(), store.path());
    let later = again.add(req(fx.url("/file/10"), dir.path(), StartMode::Later)).await.unwrap();
    assert!(later.rev > done.rev, "revision counter restarted after reload");
}
