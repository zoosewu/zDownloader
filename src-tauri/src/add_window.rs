//! The add-downloads dialog in the main window: it collects links (from the clipboard, the
//! browser extension, an imported list or typed in) so the user can check them before adding.
//!
//! Links wait in `Pending` until the dialog takes them: when the page loads, and whenever it
//! hears `add-items-ready`, which also opens the dialog. Nothing is lost while the page starts.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use ts_rs::TS;

use crate::commands::error::{CmdResult, CommandError};
use crate::utils::parser::parse_url_file;

const READY_EVENT: &str = "add-items-ready";

/// A link waiting for the add dialog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AddItem {
    pub url: String,
    #[serde(default)]
    pub filename: Option<String>,
    /// From the browser extension (Cookie, Referer, User-Agent): sites may need them.
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    /// The page the browser extension found the link on.
    #[serde(default)]
    pub page_url: Option<String>,
}

impl AddItem {
    pub fn url(url: String) -> Self {
        Self { url, filename: None, headers: Vec::new(), page_url: None }
    }
}

#[derive(Default)]
pub struct Pending(Mutex<Vec<AddItem>>);

/// Bring the main window forward with the add dialog open and `items` added to its list
/// (none: just open it).
pub async fn open(app: &AppHandle, items: Vec<AddItem>) -> CmdResult<()> {
    app.state::<Pending>().0.lock().unwrap().extend(items);
    // From the tray or another app: the dialog is in this window.
    crate::app_window::show(app);
    app.emit(READY_EVENT, ()).map_err(|e| CommandError::new("window", format!("cannot open the add dialog: {e}")))
}

/// Open the add dialog (`items` may be empty).
#[tauri::command]
pub async fn add_window_open(items: Vec<AddItem>, app: AppHandle) -> CmdResult<()> {
    open(&app, items).await
}

/// The links waiting for the add dialog; each is handed out once.
#[tauri::command]
pub fn add_window_take(pending: State<'_, Pending>) -> Vec<AddItem> {
    std::mem::take(&mut *pending.0.lock().unwrap())
}

/// Read a text file, one link per line (blank lines and `#` comments skipped), into the add
/// dialog. Returns how many links were read.
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
        let json = serde_json::to_value(AddItem { filename: Some("a".into()), page_url: Some("https://x/".into()), ..item }).unwrap();
        assert_eq!(json, serde_json::json!({ "url": "https://x/a.zip", "filename": "a", "headers": [], "pageUrl": "https://x/" }));
    }
}
