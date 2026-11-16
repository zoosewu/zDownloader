//! The add-downloads window: one window that collects links (from the clipboard, the browser
//! extension, an imported list or typed in) so the user can check them before adding.
//!
//! Links wait in `Pending` until the window takes them: a new window takes them when it loads,
//! an open one when it hears `add-items-ready`. Nothing is lost while the window starts.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};
use ts_rs::TS;

use crate::commands::error::{CmdResult, CommandError};
use crate::commands::settings::SettingsState;
use crate::utils::parser::parse_url_file;

pub const ADD_WINDOW: &str = "add";
const READY_EVENT: &str = "add-items-ready";

/// A link waiting in the add window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AddItem {
    pub url: String,
    #[serde(default)]
    pub filename: Option<String>,
    /// From the browser extension (Cookie, Referer, User-Agent): sites may need them.
    #[serde(default)]
    pub headers: Vec<(String, String)>,
}

impl AddItem {
    pub fn url(url: String) -> Self {
        Self { url, filename: None, headers: Vec::new() }
    }
}

#[derive(Default)]
pub struct Pending(Mutex<Vec<AddItem>>);

/// Show the add window with `items` added to its list (none: just show it).
pub async fn open(app: &AppHandle, items: Vec<AddItem>) -> CmdResult<()> {
    app.state::<Pending>().0.lock().unwrap().extend(items);
    if let Some(window) = app.get_webview_window(ADD_WINDOW) {
        let _ = app.emit(READY_EVENT, ());
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }
    let title = match app.state::<SettingsState>().settings.lock().await.language.as_str() {
        "en" => "Add downloads",
        "ja" => "ダウンロードを追加",
        _ => "新增下載",
    };
    WebviewWindowBuilder::new(app, ADD_WINDOW, WebviewUrl::App("index.html".into()))
        .title(title)
        .inner_size(860.0, 560.0)
        .min_inner_size(560.0, 360.0)
        .center()
        // Tauri's own handler only reports dropped files; the page handles links dragged from a
        // browser and dropped text files itself.
        .disable_drag_drop_handler()
        .build()
        .map_err(|e| CommandError::new("window", format!("cannot open the add window: {e}")))?;
    Ok(())
}

/// Open the add window (`items` may be empty). Async: creating a window from a synchronous
/// command deadlocks on Windows.
#[tauri::command]
pub async fn add_window_open(items: Vec<AddItem>, app: AppHandle) -> CmdResult<()> {
    open(&app, items).await
}

/// The links waiting for the add window; each is handed out once.
#[tauri::command]
pub fn add_window_take(pending: State<'_, Pending>) -> Vec<AddItem> {
    std::mem::take(&mut *pending.0.lock().unwrap())
}

/// Read a text file, one link per line (blank lines and `#` comments skipped), into the add
/// window. Returns how many links were read.
#[tauri::command]
pub async fn add_window_import(file_path: String, app: AppHandle) -> CmdResult<usize> {
    let urls = parse_url_file(std::path::Path::new(&file_path)).map_err(CommandError::invalid)?;
    let count = urls.len();
    open(&app, urls.into_iter().map(AddItem::url).collect()).await?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_cross_the_seam_in_camel_case() {
        let item: AddItem = serde_json::from_value(serde_json::json!({ "url": "https://x/a.zip" })).unwrap();
        assert_eq!(item, AddItem::url("https://x/a.zip".into()));
        let json = serde_json::to_value(AddItem { filename: Some("a".into()), ..item }).unwrap();
        assert_eq!(json, serde_json::json!({ "url": "https://x/a.zip", "filename": "a", "headers": [] }));
    }
}
