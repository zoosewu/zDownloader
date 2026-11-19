//! Languages the user adds or overrides: `<app data>/locales/<code>.json`, the same keys as the
//! built-in `src/locales/*.json`. The page parses and applies them (missing keys fall back to
//! English); a file named after a built-in language overrides only the keys it has.

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;
use ts_rs::TS;

use super::error::{CmdResult, CommandError};

/// Written into the folder each time it is opened, so it always lists every current key.
/// Files starting with `_` are not loaded.
const TEMPLATE_NAME: &str = "_template.json";
const ENGLISH: &str = include_str!("../../../src/locales/en.json");
const README: &str = "Copy this file to <language code>.json (fr.json, pt-BR.json; zh-TW.json changes the built-in \
    Chinese) and translate the values; keys you leave out show in English. _name is the language's name in the \
    list (empty: what the system calls it). Choose the language in Settings; reopen the list to pick up changes.";

/// The English texts with `_readme` and `_name` first.
fn template() -> String {
    let body = ENGLISH.trim_start().strip_prefix('{').expect("en.json is an object");
    let readme = serde_json::to_string(README).unwrap();
    format!("{{\n  \"_readme\": {readme},\n  \"_name\": \"\",{body}")
}

/// One `<code>.json` file, unparsed: the page reports a file it cannot read.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct UserLocale {
    /// The file name without `.json`, e.g. `fr` or `zh-TW`.
    pub code: String,
    pub text: String,
}

pub fn locales_dir() -> PathBuf {
    crate::storage::paths::get_app_data_dir().join("locales")
}

/// `en`, `fr`, `zh-TW`, `pt-BR`, `sr-Latn`: a BCP 47 language tag.
fn is_code(code: &str) -> bool {
    let mut parts = code.split('-');
    let lang = parts.next().unwrap_or_default();
    (2..=3).contains(&lang.len())
        && lang.chars().all(|c| c.is_ascii_alphabetic())
        && parts.all(|p| (2..=8).contains(&p.len()) && p.chars().all(|c| c.is_ascii_alphanumeric()))
}

fn read(dir: &Path) -> Vec<UserLocale> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut locales: Vec<UserLocale> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .filter_map(|p| {
            let code = p.file_stem()?.to_str()?.to_string();
            if !is_code(&code) {
                return None;
            }
            let text = match std::fs::read_to_string(&p) {
                Ok(text) => text,
                Err(e) => {
                    log::warn!("cannot read {}: {e}", p.display());
                    return None;
                }
            };
            Some(UserLocale { code, text })
        })
        .collect();
    locales.sort_by(|a, b| a.code.cmp(&b.code));
    locales
}

/// The user's language files.
#[tauri::command]
pub async fn locales_list() -> Vec<UserLocale> {
    read(&locales_dir())
}

/// Open the language folder, creating it with an up-to-date English template to copy from.
#[tauri::command]
pub async fn locales_open_folder(app: AppHandle) -> CmdResult<()> {
    let dir = locales_dir();
    let io = |e: std::io::Error| CommandError::new("io", format!("{}: {e}", dir.display()));
    std::fs::create_dir_all(&dir).map_err(io)?;
    std::fs::write(dir.join(TEMPLATE_NAME), template()).map_err(io)?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(|e| CommandError::new("io", format!("cannot open {}: {e}", dir.display())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_codes() {
        for ok in ["en", "fr", "zh-TW", "pt-BR", "sr-Latn", "es-419", "fil"] {
            assert!(is_code(ok), "{ok}");
        }
        for bad in ["_template", "e", "english", "zh_TW", "zh-", "en-a", "../en", "en.bak"] {
            assert!(!is_code(bad), "{bad}");
        }
    }

    #[test]
    fn only_language_files_are_read() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("fr.json"), r#"{"common":{"save":"Enregistrer"}}"#).unwrap();
        std::fs::write(dir.path().join("zh-TW.json"), "{}").unwrap();
        std::fs::write(dir.path().join(TEMPLATE_NAME), template()).unwrap();
        std::fs::write(dir.path().join("notes.txt"), "x").unwrap();
        std::fs::write(dir.path().join("fr.json.bak"), "x").unwrap();
        let codes: Vec<_> = read(dir.path()).into_iter().map(|l| l.code).collect();
        assert_eq!(codes, ["fr", "zh-TW"]);
        assert!(read(&dir.path().join("missing")).is_empty());
    }

    #[test]
    fn the_template_is_english_with_notes() {
        let template: serde_json::Value = serde_json::from_str(&template()).unwrap();
        let english: serde_json::Value = serde_json::from_str(ENGLISH).unwrap();
        assert_eq!(template["_name"], "");
        assert!(template["_readme"].as_str().unwrap().contains("<language code>.json"));
        let mut rest = template.as_object().unwrap().clone();
        rest.remove("_readme");
        rest.remove("_name");
        assert_eq!(serde_json::Value::Object(rest), english);
    }

    #[test]
    fn serializes_camel_case() {
        let json = serde_json::to_value(UserLocale { code: "fr".into(), text: "{}".into() }).unwrap();
        assert_eq!(json, serde_json::json!({ "code": "fr", "text": "{}" }));
    }
}
