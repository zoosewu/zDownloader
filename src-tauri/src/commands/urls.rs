//! What the app can tell about a URL before downloading it: for the add-download page and the
//! clipboard watcher.

use serde::Serialize;
use tauri::State;
use ts_rs::TS;
use zoo_marketplace::InstalledInfo;

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
}

#[tauri::command]
pub fn urls_inspect(url: String, modules: State<'_, SiteModules>) -> UrlInfo {
    let module = modules.resolver.module_for(&url).and_then(|m| modules.market.installed().into_iter().find(|i| i.key == m.module.id));
    inspect(&url, module)
}

fn inspect(url: &str, module: Option<InstalledInfo>) -> UrlInfo {
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
    UrlInfo { downloadable: module.is_some() || known_type, filename, module }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_urls_name_their_file() {
        let info = inspect("https://cdn.example/files/My%20Movie.MKV?token=1", None);
        assert_eq!(info.filename.as_deref(), Some("My Movie.MKV"));
        assert!(info.downloadable);

        let page = inspect("https://example.com/watch.php?v=1", None);
        assert_eq!(page.filename.as_deref(), Some("watch.php"));
        assert!(!page.downloadable, "a web page is not offered as a download");
    }

    #[test]
    fn urls_without_a_file_name_have_none() {
        for url in ["https://example.com/", "https://example.com/share/abc", "https://example.com/.hidden", "ftp://x/a.zip", "not a url"] {
            let info = inspect(url, None);
            assert_eq!((info.filename, info.downloadable), (None, false), "{url}");
        }
    }

    #[test]
    fn serializes_camel_case() {
        let json = serde_json::to_value(inspect("https://x/a.zip", None)).unwrap();
        assert_eq!(json, serde_json::json!({ "filename": "a.zip", "module": null, "downloadable": true }));
    }
}
