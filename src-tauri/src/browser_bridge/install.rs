//! Helping the user install the browser extension: from the Edge Add-ons page, or by loading the
//! folder the app ships (bundle resources) on a browser's extensions page with developer mode on
//! ("Load unpacked"): browsers do not let a program install an extension by itself.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;
use ts_rs::TS;

use crate::commands::error::{CmdResult, CommandError};

/// Where to load the extension from, and the browsers that can.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionSetup {
    pub folder: String,
    pub browsers: Vec<Browser>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Browser {
    /// `edge` or `chrome`.
    pub id: String,
    pub name: String,
}

struct Known {
    id: &'static str,
    name: &'static str,
    /// The extensions page; copied for the user to paste.
    page: &'static str,
    #[cfg_attr(not(windows), allow(dead_code))]
    windows_exe: &'static str,
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    mac_app: &'static str,
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    linux_commands: &'static [&'static str],
}

const BROWSERS: [Known; 2] = [
    Known {
        id: "edge",
        name: "Microsoft Edge",
        page: "edge://extensions",
        windows_exe: "msedge.exe",
        mac_app: "/Applications/Microsoft Edge.app",
        linux_commands: &["microsoft-edge", "microsoft-edge-stable"],
    },
    Known {
        id: "chrome",
        name: "Google Chrome",
        page: "chrome://extensions",
        windows_exe: "chrome.exe",
        mac_app: "/Applications/Google Chrome.app",
        linux_commands: &["google-chrome", "google-chrome-stable"],
    },
];

/// The extension's Edge Add-ons page (its ID is `zoo_native_host::EXTENSION_IDS[1]`).
const STORE_PAGE: &str = "https://microsoftedge.microsoft.com/addons/detail/bhgmngdlfelamchfhlahkkclffhpgfma";

#[tauri::command]
pub fn extension_setup(app: AppHandle) -> CmdResult<ExtensionSetup> {
    let folder = folder(&app)?;
    let browsers = BROWSERS
        .iter()
        .filter(|b| program(b).is_some())
        .map(|b| Browser { id: b.id.into(), name: b.name.into() })
        .collect();
    Ok(ExtensionSetup { folder: folder.display().to_string(), browsers })
}

/// Bring `browser` up with its extensions page address (`edge://extensions`) on the clipboard,
/// and return that address. Browsers refuse to open their own pages for another program (Edge
/// opens an empty tab instead), so the user pastes it into the address bar.
#[tauri::command]
pub async fn extension_open_page(browser: String, app: AppHandle) -> CmdResult<String> {
    let known = BROWSERS.iter().find(|b| b.id == browser).ok_or_else(|| CommandError::invalid(format!("unknown browser {browser:?}")))?;
    let exe = program(known).ok_or_else(|| CommandError::new("notFound", format!("{} is not installed", known.name)))?;
    crate::commands::clipboard::clipboard_copy(known.page.into(), app).await.map_err(|e| CommandError::new("io", e))?;
    launch(&exe, None).map_err(|e| CommandError::new("io", format!("cannot start {}: {e}", known.name)))?;
    Ok(known.page.into())
}

/// Open the store page in `browser`, or in the default browser when none is given.
#[tauri::command]
pub fn extension_open_store(browser: Option<String>, app: AppHandle) -> CmdResult<()> {
    let Some(browser) = browser else {
        return app.opener().open_url(STORE_PAGE, None::<&str>).map_err(|e| CommandError::new("io", format!("cannot open {STORE_PAGE}: {e}")));
    };
    let known = BROWSERS.iter().find(|b| b.id == browser).ok_or_else(|| CommandError::invalid(format!("unknown browser {browser:?}")))?;
    let exe = program(known).ok_or_else(|| CommandError::new("notFound", format!("{} is not installed", known.name)))?;
    launch(&exe, Some(STORE_PAGE)).map_err(|e| CommandError::new("io", format!("cannot start {}: {e}", known.name)))
}

/// Show the extension folder, to pick in "Load unpacked".
#[tauri::command]
pub fn extension_show_folder(app: AppHandle) -> CmdResult<()> {
    let manifest = folder(&app)?.join("manifest.json");
    app.opener()
        .reveal_item_in_dir(&manifest)
        .map_err(|e| CommandError::new("io", format!("cannot show {}: {e}", manifest.display())))
}

/// The bundled extension; in development builds the source folder, so edits show after a
/// Reload on the extensions page.
fn folder(app: &AppHandle) -> CmdResult<PathBuf> {
    let source = cfg!(debug_assertions).then(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../browser-extension"));
    let bundled = app.path().resource_dir().ok().map(|dir| dir.join("browser-extension"));
    [source, bundled]
        .into_iter()
        .flatten()
        .find(|dir| dir.join("manifest.json").is_file())
        .map(|dir| crate::site_modules::plain_path(std::path::absolute(&dir).unwrap_or(dir)))
        .ok_or_else(|| CommandError::new("notFound", "the browser extension is missing from the app's files"))
}

/// The installed browser's program (Windows registers it under App Paths).
#[cfg(windows)]
fn program(browser: &Known) -> Option<PathBuf> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;
    let key = format!(r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\{}", browser.windows_exe);
    [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE].into_iter().find_map(|root| {
        let path: String = RegKey::predef(root).open_subkey(&key).ok()?.get_value("").ok()?;
        let path = PathBuf::from(path.trim_matches('"'));
        path.is_file().then_some(path)
    })
}

#[cfg(target_os = "macos")]
fn program(browser: &Known) -> Option<PathBuf> {
    let app = PathBuf::from(browser.mac_app);
    app.is_dir().then_some(app)
}

#[cfg(target_os = "linux")]
fn program(browser: &Known) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).flat_map(|dir| browser.linux_commands.iter().map(move |c| dir.join(c))).find(|p| p.is_file())
}

#[cfg(not(target_os = "macos"))]
fn launch(exe: &Path, url: Option<&str>) -> std::io::Result<()> {
    Command::new(exe).args(url).spawn().map(drop)
}

#[cfg(target_os = "macos")]
fn launch(app: &Path, url: Option<&str>) -> std::io::Result<()> {
    Command::new("open").arg("-a").arg(app).args(url).spawn().map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_store_page_is_the_listed_extension() {
        assert!(STORE_PAGE.ends_with(&format!("/{}", zoo_native_host::EXTENSION_IDS[1])));
    }

    /// The bundle carries every file the extension loads (not its tests or README).
    #[test]
    fn the_bundle_carries_the_whole_extension() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let conf: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root.join("tauri.conf.json")).unwrap()).unwrap();
        let resources = conf["bundle"]["resources"].as_object().unwrap();
        let ext = root.join("../browser-extension");
        let mut missing = Vec::new();
        let mut stack = vec![ext.clone()];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                let rel = path.strip_prefix(&ext).unwrap().to_string_lossy().replace('\\', "/");
                if rel == "test" || rel == "node_modules" || rel == "README.md" {
                    continue;
                }
                if path.is_dir() {
                    stack.push(path);
                } else if resources.get(&format!("../browser-extension/{rel}")).and_then(|v| v.as_str()) != Some(&format!("browser-extension/{rel}")) {
                    missing.push(rel);
                }
            }
        }
        assert!(missing.is_empty(), "add to bundle.resources in tauri.conf.json: {missing:?}");
    }
}
