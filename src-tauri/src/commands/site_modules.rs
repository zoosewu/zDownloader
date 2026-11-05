//! Site-module commands: marketplaces, installed modules, captchas. Thin calls into
//! `zoo_marketplace::Marketplace`; every change is applied to downloads at once.

use std::collections::BTreeMap;

use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};
use zoo_marketplace::{InstalledInfo, PlanSummary, SourceInfo};

use crate::commands::error::{CmdResult, CommandError};
use crate::site_modules::{self, CaptchaRequest, SiteModules, MODULES_EVENT};

/// Apply the marketplace's state to downloads and tell the frontend.
fn changed(app: &AppHandle, modules: &SiteModules) {
    modules.sync();
    let _ = app.emit(MODULES_EVENT, ());
}

#[tauri::command]
pub fn modules_list(modules: State<'_, SiteModules>) -> Vec<InstalledInfo> {
    modules.market.installed()
}

#[tauri::command]
pub fn modules_sources(modules: State<'_, SiteModules>) -> Vec<SourceInfo> {
    modules.market.sources()
}

/// `owner/repo`, an https:// catalog URL, a git URL or a folder path.
#[tauri::command]
pub async fn modules_add_source(location: String, app: AppHandle, modules: State<'_, SiteModules>) -> CmdResult<SourceInfo> {
    let source = modules.market.add_source(&location).await?;
    changed(&app, &modules);
    Ok(source)
}

#[tauri::command]
pub fn modules_remove_source(name: String, app: AppHandle, modules: State<'_, SiteModules>) -> CmdResult<()> {
    modules.market.remove_source(&name)?;
    changed(&app, &modules);
    Ok(())
}

/// Fetch every catalog again and look for updates.
#[tauri::command]
pub async fn modules_refresh(app: AppHandle, modules: State<'_, SiteModules>) -> CmdResult<Vec<SourceInfo>> {
    let sources = modules.market.refresh().await;
    changed(&app, &modules);
    Ok(sources)
}

/// Download and verify a module (new or an update); `modules_install` commits it once the
/// user accepted the permissions in the summary.
#[tauri::command]
pub async fn modules_plan_install(key: String, modules: State<'_, SiteModules>) -> CmdResult<PlanSummary> {
    let plan = modules.market.plan_install(&key).await?;
    let summary = plan.summary.clone();
    modules.keep_plan(plan);
    Ok(summary)
}

#[tauri::command]
pub fn modules_install(key: String, app: AppHandle, modules: State<'_, SiteModules>) -> CmdResult<InstalledInfo> {
    let plan = modules
        .take_plan(&key)
        .ok_or_else(|| CommandError::invalid(format!("{key} was not prepared for installing; plan it first")))?;
    let info = modules.market.install(plan)?;
    changed(&app, &modules);
    Ok(info)
}

/// The user declined the permissions.
#[tauri::command]
pub fn modules_cancel_install(key: String, modules: State<'_, SiteModules>) {
    modules.take_plan(&key);
}

#[tauri::command]
pub fn modules_remove(key: String, app: AppHandle, modules: State<'_, SiteModules>) -> CmdResult<()> {
    modules.market.remove(&key)?;
    changed(&app, &modules);
    Ok(())
}

#[tauri::command]
pub fn modules_set_enabled(key: String, enabled: bool, app: AppHandle, modules: State<'_, SiteModules>) -> CmdResult<()> {
    modules.market.set_enabled(&key, enabled)?;
    changed(&app, &modules);
    Ok(())
}

/// Move a module to `index` in the priority order (the first matching module resolves).
#[tauri::command]
pub fn modules_move(key: String, index: usize, app: AppHandle, modules: State<'_, SiteModules>) -> CmdResult<()> {
    modules.market.move_to(&key, index)?;
    changed(&app, &modules);
    Ok(())
}

/// Store credential values; `null` deletes one.
#[tauri::command]
pub fn modules_set_credentials(
    key: String,
    values: BTreeMap<String, Option<String>>,
    app: AppHandle,
    modules: State<'_, SiteModules>,
) -> CmdResult<InstalledInfo> {
    let info = modules.market.set_credentials(&key, values)?;
    changed(&app, &modules);
    Ok(info)
}

#[tauri::command]
pub fn captcha_pending(modules: State<'_, SiteModules>) -> Vec<CaptchaRequest> {
    modules.captchas.pending()
}

/// Answer a captcha; `null` declines it.
#[tauri::command]
pub fn captcha_answer(id: String, answer: Option<String>, modules: State<'_, SiteModules>) {
    modules.captchas.answer(&id, answer);
}

/// Open the site's page in a window of its own for a reCAPTCHA. The token is taken from a
/// navigation the page script makes once it is solved; closing the window declines.
#[tauri::command]
pub fn captcha_open_window(id: String, app: AppHandle, modules: State<'_, SiteModules>) -> CmdResult<()> {
    let label = site_modules::captcha_window(&id);
    if let Some(window) = app.get_webview_window(&label) {
        let _ = window.set_focus();
        return Ok(());
    }
    let page = modules.captchas.recaptcha_page(&id).ok_or_else(|| CommandError::new("notFound", "no reCAPTCHA is waiting"))?;
    let url: tauri::Url = page.parse().map_err(|e| CommandError::invalid(format!("captcha page {page}: {e}")))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(CommandError::invalid("captcha pages must be http(s)"));
    }
    let title = url.host_str().unwrap_or("captcha").to_string();

    let (captchas, answered) = (modules.captchas.clone(), id.clone());
    let window = WebviewWindowBuilder::new(&app, &label, WebviewUrl::External(url))
        .title(title)
        .inner_size(480.0, 720.0)
        .initialization_script(site_modules::RECAPTCHA_SCRIPT)
        .on_navigation(move |to| {
            if to.host_str() != Some(site_modules::TOKEN_HOST) {
                return true;
            }
            let token = to.query_pairs().find(|(k, _)| k == "token").map(|(_, v)| v.into_owned());
            // Answering closes the captcha, which closes this window.
            captchas.answer(&answered, token);
            false
        })
        .build()
        .map_err(|e| CommandError::new("window", format!("cannot open the captcha window: {e}")))?;
    let captchas = modules.captchas.clone();
    window.on_window_event(move |event| {
        if matches!(event, tauri::WindowEvent::Destroyed) {
            captchas.answer(&id, None);
        }
    });
    Ok(())
}
