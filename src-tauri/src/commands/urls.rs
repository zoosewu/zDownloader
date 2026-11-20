//! What the app can tell about a URL before downloading it: for the add-download page and the
//! clipboard watcher.

use std::collections::BTreeMap;

use serde::Serialize;
use tauri::State;
use ts_rs::TS;
use zoo_marketplace::InstalledInfo;

use crate::commands::error::CmdResult;
use crate::commands::settings::SettingsState;
use crate::site_modules::SiteModules;

/// File types worth offering as a download when their URL shows up on the clipboard.
const DOWNLOAD_TYPES: &[&str] = &[
    "zip", "rar", "7z", "tar", "gz", "xz", "bz2", "iso", "img", "dmg", "exe", "msi", "apk", "deb", "rpm", "pdf", "epub",
    "mp4", "mkv", "avi", "mov", "webm", "mp3", "flac", "wav", "m4a",
];

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct UrlInfo {
    /// The name the file would get, when the URL's path ends in one (`.../movie.mp4`).
    pub filename: Option<String>,
    /// The installed module that resolves the URL.
    pub module: Option<InstalledInfo>,
    /// Worth offering as a download: a module resolves it or it names a common file type.
    pub downloadable: bool,
    /// Start it at once when copied, instead of showing it in the add window: the switch for
    /// copied links is on and the resolving module, if any, allows it.
    pub auto_start: bool,
}

/// One file behind a link (see `urls_list`).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ListedFile {
    /// A link for just this file: what its task downloads.
    pub url: String,
    pub filename: Option<String>,
    pub size: Option<u64>,
    /// Folder inside the share (`a/b`); `None` at the top.
    pub folder: Option<String>,
    /// What the user answered the module while it listed the link (an extraction code): give
    /// it to `downloads_add` so the download does not ask again. Secret answers are not here.
    pub answers: BTreeMap<String, String>,
}

/// The files behind `url`: several for a shared folder whose module lists it, otherwise one
/// (the link itself). Runs the module, so it may take a few seconds and may ask the user.
#[tauri::command]
pub async fn urls_list(url: String, modules: State<'_, SiteModules>) -> CmdResult<Vec<ListedFile>> {
    Ok(list(&url, &modules).await?)
}

async fn list(url: &str, modules: &SiteModules) -> Result<Vec<ListedFile>, zoo_core::CoreError> {
    let (items, answers) = modules.resolver.list(url).await?;
    let answers = answers.saved().clone();
    Ok(items
        .into_iter()
        .map(|i| ListedFile { url: i.url, filename: i.filename, size: i.size, folder: i.folder, answers: answers.clone() })
        .collect())
}

#[tauri::command]
pub async fn urls_inspect(url: String, modules: State<'_, SiteModules>, settings: State<'_, SettingsState>) -> CmdResult<UrlInfo> {
    let allowed = settings.settings.lock().await.auto_start_copied;
    Ok(inspect(&url, module_for(&url, &modules), allowed))
}

/// The file to start at once for a download the browser extension sends: the switch for the
/// browser is on, the link's module (if any) allows it, and the link is one file. A shared
/// folder waits in the add window for the user to pick its files, and so does a link its
/// module cannot list (the add window shows why).
pub async fn starts_at_once(url: &str, modules: &SiteModules, settings: &SettingsState) -> Option<ListedFile> {
    let allowed = settings.settings.lock().await.auto_start_browser;
    if !allowed || !module_for(url, modules).is_none_or(|m| m.auto_start) {
        return None;
    }
    let mut files = list(url, modules).await.ok()?;
    if files.len() == 1 { files.pop() } else { None }
}

fn module_for(url: &str, modules: &SiteModules) -> Option<InstalledInfo> {
    let key = modules.resolver.module_for(url)?.module.id;
    modules.market.installed().into_iter().find(|i| i.key == key)
}

/// `auto_start_allowed`: the switch for where the link came from; without it nothing starts by
/// itself.
fn inspect(url: &str, module: Option<InstalledInfo>, auto_start_allowed: bool) -> UrlInfo {
    let filename = match tauri::Url::parse(url) {
        Ok(parsed) if matches!(parsed.scheme(), "http" | "https") => {
            let name = zoo_core::naming::filename_for(None, parsed.as_str());
            name.rsplit_once('.').is_some_and(|(stem, ext)| !stem.is_empty() && !ext.is_empty()).then_some(name)
        }
        _ => None,
    };
    let known_type = filename
        .as_deref()
        .and_then(|name| name.rsplit_once('.'))
        .is_some_and(|(_, ext)| DOWNLOAD_TYPES.contains(&ext.to_ascii_lowercase().as_str()));
    let auto_start = auto_start_allowed && module.as_ref().is_none_or(|m| m.auto_start);
    UrlInfo { downloadable: module.is_some() || known_type, filename, module, auto_start }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_urls_name_their_file() {
        let info = inspect("https://cdn.example/files/My%20Movie.MKV?token=1", None, true);
        assert_eq!(info.filename.as_deref(), Some("My Movie.MKV"));
        assert!(info.downloadable);

        let page = inspect("https://example.com/watch.php?v=1", None, true);
        assert_eq!(page.filename.as_deref(), Some("watch.php"));
        assert!(!page.downloadable, "a web page is not offered as a download");
    }

    #[test]
    fn urls_without_a_file_name_have_none() {
        for url in ["https://example.com/", "https://example.com/share/abc", "https://example.com/.hidden", "ftp://x/a.zip", "not a url"] {
            let info = inspect(url, None, true);
            assert_eq!((info.filename, info.downloadable), (None, false), "{url}");
        }
    }

    #[test]
    fn serializes_camel_case() {
        let json = serde_json::to_value(inspect("https://x/a.zip", None, true)).unwrap();
        assert_eq!(json, serde_json::json!({ "filename": "a.zip", "module": null, "downloadable": true, "autoStart": true }));
        assert!(!inspect("https://x/a.zip", None, false).auto_start, "the app-wide switch decides first");
    }
}
