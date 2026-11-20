//! End to end: DownloadManager → ModuleResolver → module host → site module → fixture site.

mod common;

use std::sync::Arc;
use std::time::Duration;

use common::{loopback_host, module_allowing, Resp, Server};
use tempfile::TempDir;
use zoo_core::connection::Direct;
use zoo_core::{AddRequest, DownloadManager, Event, Phase, StartMode, Task, TaskId, TaskState};
use zoo_module_client::resolver::{InstalledModule, ModuleResolver};
use zoo_module_client::{Limits, NoInteraction};

const FILE: &[u8] = b"the real file contents";

/// A share site: the page sets a session cookie and links to the file; the file needs both
/// the cookie and a Referer.
async fn share_site() -> Server {
    Server::start(|req| match req.path.as_str() {
        "/page/abc" => Resp::ok(r#"<html><h1 class="name">holiday.mp4</h1><a id="dl" href="/dl/abc?token=t1">Download</a></html>"#)
            .header("Set-Cookie", "sid=42; Path=/"),
        p if p.starts_with("/dl/abc") => {
            let cookie_ok = req.headers.get("cookie").map(String::as_str) == Some("sid=42");
            let referer_ok = req.headers.get("referer").is_some_and(|r| r.ends_with("/page/abc"));
            if cookie_ok && referer_ok {
                Resp::ok(FILE)
            } else {
                Resp { status: 403, headers: vec![], body: b"no session".to_vec() }
            }
        }
        "/plain.bin" => Resp::ok(FILE),
        _ => Resp { status: 404, headers: vec![], body: vec![] },
    })
    .await
}

const MODULE: &str = r#"
import { defineModule } from 'zoo-module';
export default defineModule({
  async resolve(url, zoo) {
    if (url.includes('slow')) {
      await zoo.wait(1, 'countdown');
      await fetch(new URL('/after-wait', url));
    }
    const page = await fetch(url);
    const doc = new DOMParser().parseFromString(await page.text(), 'text/html');
    const href = doc.getElementById('dl')?.getAttribute('href');
    if (!href) throw zoo.fail('no download link');
    return { url: new URL(href, page.url).href, filename: doc.querySelector('h1.name').textContent, headers: { Referer: page.url } };
  },
});
"#;

fn manager(dir: &TempDir, site: &Server) -> (DownloadManager, Arc<ModuleResolver>) {
    let resolver = Arc::new(ModuleResolver::new(Arc::new(loopback_host()), Arc::new(NoInteraction)));
    resolver.set_modules(vec![InstalledModule {
        module: module_allowing(dir, "share", MODULE, &["127.0.0.1"]),
        url_patterns: vec![format!("http://{}/page/*", site.addr)],
        url_regex: None,
        limits: Limits::default(),
    }]);
    let store = dir.path().join("store");
    std::fs::create_dir_all(&store).unwrap();
    let m = DownloadManager::start_with(Default::default(), &store, Arc::new(Direct::new().unwrap()), resolver.clone()).unwrap();
    (m, resolver)
}

fn add(dir: &TempDir, url: String) -> AddRequest {
    AddRequest { url, filename: None, dir: dir.path().to_path_buf(), start: StartMode::Now, headers: Vec::new(), tags: None, answers: Default::default() }
}

async fn wait_until(m: &DownloadManager, id: TaskId, pred: impl Fn(&Task) -> bool) -> Task {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
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
async fn a_share_page_downloads_through_its_module() {
    let site = share_site().await;
    let dir = TempDir::new().unwrap();
    let (m, _) = manager(&dir, &site);
    let mut events = m.subscribe();

    let id = m.add(add(&dir, site.url("/page/abc"))).await.unwrap().id;
    let task = wait_until(&m, id, |t| matches!(t.state, TaskState::Completed | TaskState::Failed)).await;

    assert_eq!(task.state, TaskState::Completed, "{:?}", task.error);
    assert_eq!(task.name, "holiday.mp4");
    assert_eq!(std::fs::read(task.final_path.unwrap()).unwrap(), FILE);
    let resolving = std::iter::from_fn(|| events.try_recv().ok()).any(|e| {
        matches!(e, Event::Upserted(t) if t.phase == Some(Phase::Resolving { module: Some("share@test".into()) }))
    });
    assert!(resolving, "the task showed which module resolved it");
}

#[tokio::test]
async fn urls_no_module_claims_download_directly() {
    let site = share_site().await;
    let dir = TempDir::new().unwrap();
    let (m, resolver) = manager(&dir, &site);
    assert!(resolver.module_for(&site.url("/plain.bin")).is_none());

    let id = m.add(add(&dir, site.url("/plain.bin"))).await.unwrap().id;
    let task = wait_until(&m, id, |t| t.state == TaskState::Completed).await;
    assert_eq!(std::fs::read(task.final_path.unwrap()).unwrap(), FILE);
}

#[tokio::test]
async fn pausing_stops_the_module_in_the_host() {
    let site = share_site().await;
    let dir = TempDir::new().unwrap();
    let (m, _) = manager(&dir, &site);

    let id = m.add(add(&dir, site.url("/page/abc?slow"))).await.unwrap().id;
    let waiting = wait_until(&m, id, |t| matches!(t.phase, Some(Phase::Waiting { .. }))).await;
    assert!(matches!(&waiting.phase, Some(Phase::Waiting { reason: Some(r), .. }) if r == "countdown"));
    m.pause(id).await.unwrap();

    // Had the call kept running, it would fetch /after-wait once the countdown ended.
    tokio::time::sleep(Duration::from_millis(1500)).await;
    let paths: Vec<_> = site.requests().iter().map(|r| r.path.clone()).collect();
    let task = m.list().await.unwrap().into_iter().find(|t| t.id == id).unwrap();
    assert!(!paths.iter().any(|p| p == "/after-wait"), "the cancelled call went on: {paths:?} {task:#?}");
}

/// MODULE's resolve plus `list`: the page's file under two URLs, or (`?stray`) another site's.
fn listing_module() -> String {
    let list = r#"
  list(url) {
    if (url.includes('stray')) return [{ url: 'https://elsewhere.example/x', filename: 'x' }];
    return [
      { url: url + '#path=one', filename: 'one.mp4' },
      { url: url + '#path=two', filename: 'two.mp4', folder: 'sub' },
    ];
  },
  async resolve("#;
    MODULE.replacen("  async resolve(", list, 1)
}

#[tokio::test]
async fn listed_files_resolve_and_download_through_the_same_module() {
    let site = share_site().await;
    let dir = TempDir::new().unwrap();
    let (m, resolver) = manager(&dir, &site);
    resolver.set_modules(vec![InstalledModule {
        module: module_allowing(&dir, "lister", &listing_module(), &["127.0.0.1"]),
        url_patterns: vec![format!("http://{}/page/*", site.addr)],
        url_regex: None,
        limits: Limits::default(),
    }]);

    let (items, _) = resolver.list(&site.url("/page/abc")).await.unwrap();
    assert_eq!(items.iter().map(|i| i.filename.as_deref().unwrap()).collect::<Vec<_>>(), ["one.mp4", "two.mp4"]);
    // Each listed URL is a task of its own that the module resolves.
    let id = m.add(add(&dir, items[1].url.clone())).await.unwrap().id;
    let task = wait_until(&m, id, |t| matches!(t.state, TaskState::Completed | TaskState::Failed)).await;
    assert_eq!(task.state, TaskState::Completed, "{:?}", task.error);
    assert_eq!(std::fs::read(task.final_path.unwrap()).unwrap(), FILE);

    let stray = resolver.list(&site.url("/page/abc?stray")).await.unwrap_err();
    assert!(stray.to_string().contains("urlPatterns do not match"), "{stray}");

    let (plain, _) = resolver.list(&site.url("/plain.bin")).await.unwrap();
    assert_eq!(plain.len(), 1);
    assert_eq!(plain[0].url, site.url("/plain.bin"));
}

/// Answers every question with "good" and keeps what it was asked.
#[derive(Default)]
struct Answering(std::sync::Mutex<Vec<zoo_module_client::Captcha>>);

impl zoo_module_client::Interaction for Answering {
    fn waiting(&self, _ms: u64, _reason: Option<&str>) {}
    fn solve_captcha(
        &self,
        challenge: zoo_module_client::Captcha,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<String>> + Send + '_>> {
        self.0.lock().unwrap().push(challenge);
        Box::pin(async { Some("good".to_string()) })
    }
}

/// A share behind an extraction code: `list` and `resolve` both need it.
const CODE_MODULE: &str = r#"
import { defineModule } from 'zoo-module';
const code = async (zoo) => {
  let pwd = await zoo.ask({ id: 'pwd', message: 'Extraction code', remember: true });
  if (pwd !== 'good') pwd = await zoo.ask({ id: 'pwd', message: 'Extraction code', remember: true, error: 'wrong code' });
  return pwd;
};
export default defineModule({
  async list(url, zoo) {
    await code(zoo);
    return [{ url: url + '#path=one', filename: 'one.bin' }];
  },
  async resolve(url, zoo) {
    await code(zoo);
    return { url: new URL('/plain.bin', url).href };
  },
});
"#;

#[tokio::test]
async fn remembered_answers_are_given_again_without_asking() {
    let site = share_site().await;
    let dir = TempDir::new().unwrap();
    let user = Arc::new(Answering::default());
    let resolver = Arc::new(ModuleResolver::new(Arc::new(loopback_host()), user.clone()));
    let mut code_module = module_allowing(&dir, "coded", CODE_MODULE, &["127.0.0.1"]);
    code_module.api_version = 3;
    resolver.set_modules(vec![InstalledModule {
        module: code_module,
        url_patterns: vec![format!("http://{}/page/*", site.addr)],
        url_regex: None,
        limits: Limits::default(),
    }]);
    let store = dir.path().join("store");
    std::fs::create_dir_all(&store).unwrap();
    let m = DownloadManager::start_with(Default::default(), &store, Arc::new(Direct::new().unwrap()), resolver.clone()).unwrap();

    // Listing asks once; the file's download gets that answer and does not ask.
    let (items, answers) = resolver.list(&site.url("/page/abc")).await.unwrap();
    assert_eq!(answers.get("pwd"), Some("good"));
    let mut req = add(&dir, items[0].url.clone());
    req.answers = answers.saved().clone();
    let id = m.add(req).await.unwrap().id;
    let task = wait_until(&m, id, |t| matches!(t.state, TaskState::Completed | TaskState::Failed)).await;
    assert_eq!(task.state, TaskState::Completed, "{:?}", task.error);
    assert_eq!(user.0.lock().unwrap().len(), 1, "asked only while listing");

    // A remembered answer that no longer works: the module asks again, saying why, and the
    // task keeps the new answer.
    let mut req = add(&dir, items[0].url.clone());
    req.answers = [("pwd".to_string(), "stale".to_string())].into();
    let id = m.add(req).await.unwrap().id;
    let task = wait_until(&m, id, |t| matches!(t.state, TaskState::Completed | TaskState::Failed)).await;
    assert_eq!(task.state, TaskState::Completed, "{:?}", task.error);
    let asked = user.0.lock().unwrap().clone();
    assert_eq!(asked.len(), 2);
    assert!(matches!(&asked[1], zoo_module_client::Captcha::Ask { error: Some(_), .. }), "{asked:?}");
    assert_eq!(task.answers.get("pwd"), Some("good"));
}
