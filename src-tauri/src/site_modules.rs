//! Shell side of site modules: the marketplace, the resolver it feeds, and the user answering
//! captchas. Commands live in `commands::site_modules`.

use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{broadcast, oneshot};
use ts_rs::TS;
use zoo_marketplace::{MarketError, Marketplace, Plan, SecretStore};
use zoo_module_client::resolver::ModuleResolver;
use zoo_module_client::{Captcha, Interaction, Secret, Supervisor};

/// Installed modules or marketplaces changed (by a command, a refresh or a local edit).
pub const MODULES_EVENT: &str = "modules-changed";
/// A `CaptchaEvent`.
pub const CAPTCHA_EVENT: &str = "captcha-event";
/// How often local (in-development) modules are re-read.
const LOCAL_RELOAD: Duration = Duration::from_secs(2);

pub struct SiteModules {
    pub market: Arc<Marketplace>,
    pub resolver: Arc<ModuleResolver>,
    pub captchas: Arc<Captchas>,
    /// Downloaded and verified, waiting for the user's consent; by module key.
    plans: Mutex<HashMap<String, Plan>>,
}

impl SiteModules {
    pub fn open(data_dir: &Path) -> Result<Self, MarketError> {
        let market = Arc::new(Marketplace::open(&data_dir.join("site-modules"), Arc::new(KeyringSecrets))?);
        let captchas = Arc::new(Captchas::default());
        let resolver = Arc::new(ModuleResolver::new(Arc::new(module_host()), captchas.clone()));
        let modules = Self { market, resolver, captchas, plans: Mutex::default() };
        modules.sync();
        Ok(modules)
    }

    /// Make downloads use the installed modules as they are now.
    pub fn sync(&self) {
        self.resolver.set_modules(self.market.resolver_modules());
    }

    pub fn keep_plan(&self, plan: Plan) {
        self.plans.lock().unwrap().insert(plan.summary.key.clone(), plan);
    }

    pub fn take_plan(&self, key: &str) -> Option<Plan> {
        self.plans.lock().unwrap().remove(key)
    }
}

/// The site-module host (`zoo-module serve`), started on first use. It ships next to the app
/// executable: Tauri's externalBin in a bundle, the workspace target directory in development.
fn module_host() -> Supervisor {
    let name = if cfg!(windows) { "zoo-module.exe" } else { "zoo-module" };
    let path = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(name)))
        .unwrap_or_else(|| name.into());
    if !path.exists() {
        log::warn!("site-module host not found at {}; build it with `cargo build -p zoo-module`", path.display());
    }
    Supervisor::new(path, ["serve"])
}

/// Register the official marketplace, check for updates if the user wants that, relay
/// captchas to the frontend and keep local modules in sync with their folders.
pub fn start(app: AppHandle, check_updates: bool) {
    let official = official_dir(&app);
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let modules = handle.state::<SiteModules>();
        match official {
            Some(dir) => {
                if let Err(e) = modules.market.set_official(&dir).await {
                    log::warn!("cannot load the official marketplace from {}: {e}", dir.display());
                }
            }
            None => log::warn!("the official marketplace is missing from the app's resources"),
        }
        if check_updates {
            modules.market.refresh().await;
        }
        modules.sync();
        let _ = handle.emit(MODULES_EVENT, ());
    });

    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut tick = tokio::time::interval(LOCAL_RELOAD);
        loop {
            tick.tick().await;
            let modules = handle.state::<SiteModules>();
            match modules.market.reload_local() {
                Ok(true) => {
                    modules.sync();
                    let _ = handle.emit(MODULES_EVENT, ());
                }
                Ok(false) => {}
                Err(e) => log::warn!("cannot reload local modules: {e}"),
            }
        }
    });

    let mut events = app.state::<SiteModules>().captchas.events.subscribe();
    tauri::async_runtime::spawn(async move {
        loop {
            match events.recv().await {
                Ok(event) => {
                    if let CaptchaEvent::Closed(id) = &event {
                        if let Some(window) = app.get_webview_window(&captcha_window(id)) {
                            let _ = window.close();
                        }
                    }
                    let _ = app.emit(CAPTCHA_EVENT, &event);
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(broadcast::error::RecvError::Closed) => return,
            }
        }
    });
}

/// `marketplace/` from the bundle's resources; the source tree in development builds.
fn official_dir(app: &AppHandle) -> Option<PathBuf> {
    let bundled = app.path().resource_dir().ok().map(|dir| dir.join("marketplace"));
    let source = cfg!(debug_assertions).then(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("marketplace"));
    [bundled, source].into_iter().flatten().find(|dir| dir.join(zoo_marketplace::catalog::FILE).is_file()).map(plain_path)
}

/// Without Windows' `\\?\` prefix (resource paths have it), so the folder reads as people type it.
fn plain_path(path: PathBuf) -> PathBuf {
    match path.to_str().and_then(|s| s.strip_prefix(r"\\?\")) {
        Some(rest) if !rest.starts_with(r"UNC\") => PathBuf::from(rest),
        _ => path,
    }
}

#[cfg(test)]
#[test]
fn verbatim_prefixes_are_dropped() {
    assert_eq!(plain_path(PathBuf::from(r"\\?\D:\app\marketplace")), PathBuf::from(r"D:\app\marketplace"));
    assert_eq!(plain_path(PathBuf::from(r"\\?\UNC\server\share")), PathBuf::from(r"\\?\UNC\server\share"));
    assert_eq!(plain_path(PathBuf::from("/opt/app")), PathBuf::from("/opt/app"));
}

/// Module credentials in the OS keychain.
struct KeyringSecrets;

const KEYRING_SERVICE: &str = "zoo-downloader.site-modules";

impl SecretStore for KeyringSecrets {
    fn get(&self, account: &str) -> Result<Option<Secret>, String> {
        match keyring::Entry::new(KEYRING_SERVICE, account).and_then(|e| e.get_password()) {
            Ok(value) => Ok(Some(Secret(value))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }
    fn set(&self, account: &str, value: &Secret) -> Result<(), String> {
        keyring::Entry::new(KEYRING_SERVICE, account).and_then(|e| e.set_password(&value.0)).map_err(|e| e.to_string())
    }
    fn delete(&self, account: &str) -> Result<(), String> {
        match keyring::Entry::new(KEYRING_SERVICE, account).and_then(|e| e.delete_password()) {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

// --- captchas ----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum CaptchaEvent {
    /// A module needs the user to solve this.
    Opened(CaptchaRequest),
    /// Answered, declined, or the download stopped waiting for it.
    Closed(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CaptchaRequest {
    pub id: String,
    pub challenge: CaptchaChallenge,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum CaptchaChallenge {
    /// Type the text in the image.
    Image { mime: String, base64: String },
    /// Solve it on the site's own page (`captcha_open_window`).
    Recaptcha { site_key: String, page_url: String },
}

impl From<Captcha> for CaptchaChallenge {
    fn from(c: Captcha) -> Self {
        match c {
            Captcha::Image { mime, base64 } => CaptchaChallenge::Image { mime, base64 },
            Captcha::Recaptcha { site_key, page_url } => CaptchaChallenge::Recaptcha { site_key, page_url },
        }
    }
}

/// A captcha waiting for the user, and where the answer goes.
type Waiting = (CaptchaRequest, oneshot::Sender<Option<String>>);

/// Captchas waiting for the user. Modules ask through `Interaction`; the frontend answers.
pub struct Captchas {
    pending: Mutex<HashMap<String, Waiting>>,
    events: broadcast::Sender<CaptchaEvent>,
    next: AtomicU64,
}

impl Default for Captchas {
    fn default() -> Self {
        Self { pending: Mutex::default(), events: broadcast::channel(64).0, next: AtomicU64::new(1) }
    }
}

impl Captchas {
    pub fn pending(&self) -> Vec<CaptchaRequest> {
        let mut list: Vec<_> = self.pending.lock().unwrap().values().map(|(r, _)| r.clone()).collect();
        list.sort_by_key(|r| r.id.parse::<u64>().unwrap_or(0));
        list
    }

    /// Answer (or decline with `None`); `false` if nothing waits under `id` any more.
    pub fn answer(&self, id: &str, answer: Option<String>) -> bool {
        match self.pending.lock().unwrap().remove(id) {
            Some((_, reply)) => reply.send(answer.filter(|a| !a.is_empty())).is_ok(),
            None => false,
        }
    }

    pub fn recaptcha_page(&self, id: &str) -> Option<String> {
        match &self.pending.lock().unwrap().get(id)?.0.challenge {
            CaptchaChallenge::Recaptcha { page_url, .. } => Some(page_url.clone()),
            CaptchaChallenge::Image { .. } => None,
        }
    }

    #[cfg(test)]
    fn subscribe(&self) -> broadcast::Receiver<CaptchaEvent> {
        self.events.subscribe()
    }
}

/// Announces `Closed` however the wait ends, including the download being paused.
struct Closing<'a> {
    captchas: &'a Captchas,
    id: String,
}

impl Drop for Closing<'_> {
    fn drop(&mut self) {
        self.captchas.pending.lock().unwrap().remove(&self.id);
        let _ = self.captchas.events.send(CaptchaEvent::Closed(self.id.clone()));
    }
}

impl Interaction for Captchas {
    fn solve_captcha(&self, challenge: Captcha) -> Pin<Box<dyn Future<Output = Option<String>> + Send + '_>> {
        Box::pin(async move {
            let id = self.next.fetch_add(1, Ordering::Relaxed).to_string();
            let request = CaptchaRequest { id: id.clone(), challenge: challenge.into() };
            let (reply, answer) = oneshot::channel();
            self.pending.lock().unwrap().insert(id.clone(), (request.clone(), reply));
            let _closing = Closing { captchas: self, id };
            let _ = self.events.send(CaptchaEvent::Opened(request));
            answer.await.ok().flatten()
        })
    }
}

pub fn captcha_window(id: &str) -> String {
    format!("captcha-{id}")
}

/// Navigating here hands the token over; nothing is ever loaded from it.
pub const TOKEN_HOST: &str = "zoo-captcha.invalid";

/// Runs in the site's page: once the user solved the reCAPTCHA, navigate to `TOKEN_HOST` with
/// the token. The page gets no access to the app (the window has no capabilities).
pub const RECAPTCHA_SCRIPT: &str = r#"
(function () {
  const timer = setInterval(function () {
    let token = '';
    try {
      token = (window.grecaptcha && window.grecaptcha.getResponse && window.grecaptcha.getResponse()) || '';
    } catch (e) {}
    if (!token) {
      const field = document.querySelector('textarea[name="g-recaptcha-response"]');
      token = field ? field.value : '';
    }
    if (token) {
      clearInterval(timer);
      window.location.href = 'https://zoo-captcha.invalid/?token=' + encodeURIComponent(token);
    }
  }, 500);
})();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_captcha_waits_for_its_answer_and_always_closes() {
        let captchas = Arc::new(Captchas::default());
        let mut events = captchas.subscribe();
        let challenge = Captcha::Image { mime: "image/png".into(), base64: "AAAA".into() };

        let asking = { let c = captchas.clone(); let ch = challenge.clone(); tokio::spawn(async move { c.solve_captcha(ch).await }) };
        let CaptchaEvent::Opened(request) = events.recv().await.unwrap() else { panic!("opened first") };
        assert_eq!(captchas.pending(), vec![request.clone()]);
        assert!(captchas.answer(&request.id, Some("xk7p".into())));
        assert_eq!(asking.await.unwrap().as_deref(), Some("xk7p"));
        assert_eq!(events.recv().await.unwrap(), CaptchaEvent::Closed(request.id.clone()));
        assert!(!captchas.answer(&request.id, Some("late".into())), "answered once");

        // The download stops waiting (the future is dropped): the captcha closes by itself.
        let asking = { let c = captchas.clone(); tokio::spawn(async move { c.solve_captcha(challenge).await }) };
        let CaptchaEvent::Opened(request) = events.recv().await.unwrap() else { panic!() };
        asking.abort();
        assert_eq!(events.recv().await.unwrap(), CaptchaEvent::Closed(request.id));
        assert!(captchas.pending().is_empty());
    }

    #[test]
    fn challenges_serialize_for_the_frontend() {
        let request = CaptchaRequest {
            id: "1".into(),
            challenge: Captcha::Recaptcha { site_key: "k".into(), page_url: "https://x/".into() }.into(),
        };
        assert_eq!(
            serde_json::to_value(CaptchaEvent::Opened(request)).unwrap(),
            serde_json::json!({ "type": "opened", "data": { "id": "1", "challenge": { "kind": "recaptcha", "siteKey": "k", "pageUrl": "https://x/" } } })
        );
    }
}
