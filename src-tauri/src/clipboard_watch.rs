//! Clipboard monitoring, pushed instead of polled: the OS reports each clipboard change
//! (WM_CLIPBOARDUPDATE on Windows) and a copied link reaches the frontend as one event.
//! The clipboard is read only while monitoring is enabled in settings, and only a single
//! http(s) link is ever sent.

use crate::commands::settings::SettingsState;
use clipboard_rs::{Clipboard, ClipboardContext, ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext, ContentFormat};
use tauri::{AppHandle, Emitter, Manager};

pub const CLIPBOARD_LINK_EVENT: &str = "clipboard-link";

/// What the app itself last put on the clipboard (`clipboard_copy`): not a link to offer.
#[derive(Default)]
pub struct OwnCopy(pub std::sync::Mutex<Option<String>>);

struct Forward {
    app: AppHandle,
    clipboard: ClipboardContext,
}

impl ClipboardHandler for Forward {
    fn on_clipboard_change(&mut self) {
        // Runs on the watcher's own thread, outside the async runtime.
        if !self.app.state::<SettingsState>().settings.blocking_lock().enable_clipboard_monitoring {
            return;
        }
        let Some(text) = self.read_text() else { return };
        if self.app.state::<OwnCopy>().0.lock().unwrap().take().is_some_and(|own| own == text) {
            return;
        }
        let Some(link) = link(&text) else { return };
        if let Err(e) = self.app.emit(CLIPBOARD_LINK_EVENT, link) {
            log::warn!("failed to emit {CLIPBOARD_LINK_EVENT}: {e}");
        }
    }
}

impl Forward {
    /// The clipboard's text. Right after a change other programs (clipboard history, other
    /// watchers) often have the clipboard open, and a read then fails; retry briefly so a
    /// copied link is not missed.
    fn read_text(&self) -> Option<String> {
        for attempt in 0..10 {
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_millis(30));
            }
            match self.clipboard.get_text() {
                Ok(text) => return Some(text),
                // No text on the clipboard (an image, files): nothing to retry.
                Err(_) if !self.clipboard.has(ContentFormat::Text) && attempt > 0 => return None,
                Err(_) => {}
            }
        }
        log::warn!("clipboard changed but its text could not be read");
        None
    }
}

pub fn start(app: AppHandle) {
    let spawned = std::thread::Builder::new().name("clipboard-watch".into()).spawn(move || {
        let (clipboard, mut watcher) = match (ClipboardContext::new(), ClipboardWatcherContext::new()) {
            (Ok(c), Ok(w)) => (c, w),
            (Err(e), _) | (_, Err(e)) => {
                log::warn!("clipboard monitoring unavailable: {e}");
                return;
            }
        };
        // Dropping the shutdown handle would stop the watch; keep it for the thread's life.
        let _shutdown = watcher.add_handler(Forward { app, clipboard }).get_shutdown_channel();
        watcher.start_watch();
    });
    if let Err(e) = spawned {
        log::warn!("clipboard monitoring not started: {e}");
    }
}

/// `text` if it is exactly one http(s) URL (surrounding whitespace ignored).
fn link(text: &str) -> Option<String> {
    let text = text.trim();
    let is_url = text.len() <= 8192
        && !text.contains(char::is_whitespace)
        && ["http://", "https://"].iter().any(|scheme| text.get(..scheme.len()).is_some_and(|p| p.eq_ignore_ascii_case(scheme)));
    is_url.then(|| text.to_string())
}

#[cfg(test)]
mod tests {
    use super::link;

    #[test]
    fn only_a_single_http_link_is_sent() {
        assert_eq!(link("  https://a.example/f.zip\n").as_deref(), Some("https://a.example/f.zip"));
        assert_eq!(link("HTTP://A/x").as_deref(), Some("HTTP://A/x"));
        for not in ["", "hello", "ftp://a/x", "see https://a/x", "https://a/x https://b/y", "file:///C:/x"] {
            assert_eq!(link(not), None, "{not:?}");
        }
        assert_eq!(link(&format!("https://a/{}", "x".repeat(9000))), None);
    }
}
