//! The main window when it is not on screen: the tray icon (closing the window hides it there),
//! starting at login (hidden in the tray if the user wants), and a second launch showing the
//! running app instead of starting another one.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use serde::Deserialize;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Window, WindowEvent};
use tauri_plugin_autostart::ManagerExt;
use ts_rs::TS;

use crate::commands::error::{CmdResult, CommandError};
use crate::models::settings::AppSettings;
use crate::MAIN_WINDOW;

const TRAY: &str = "main";
/// What the login item starts the app with.
pub const AUTOSTART_ARG: &str = "--autostart";

/// Whether closing the main window hides it in the tray (the tray icon is on).
#[derive(Default)]
pub struct CloseToTray(AtomicBool);

/// The tray menu in the user's language; the page sends it once its language is known.
#[derive(Debug, Clone, PartialEq, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TrayLabels {
    pub show_hide: String,
    pub pause_all: String,
    pub resume_all: String,
    pub quit: String,
}

impl Default for TrayLabels {
    fn default() -> Self {
        Self { show_hide: "Show/Hide".into(), pause_all: "Pause all".into(), resume_all: "Resume all".into(), quit: "Quit".into() }
    }
}

#[derive(Default)]
pub struct Labels(Mutex<TrayLabels>);

pub fn show(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Into the tray: off the taskbar (and the Dock on macOS).
pub fn hide(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        let _ = window.hide();
    }
    #[cfg(target_os = "macos")]
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
}

fn toggle(app: &AppHandle) {
    let visible = app.get_webview_window(MAIN_WINDOW).is_some_and(|w| w.is_visible().unwrap_or(false));
    if visible {
        hide(app)
    } else {
        show(app)
    }
}

fn menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let labels = app.state::<Labels>().0.lock().unwrap().clone();
    Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "show_hide", labels.show_hide, true, None::<&str>)?,
            &MenuItem::with_id(app, "pause_all", labels.pause_all, true, None::<&str>)?,
            &MenuItem::with_id(app, "resume_all", labels.resume_all, true, None::<&str>)?,
            &MenuItem::with_id(app, "quit", labels.quit, true, None::<&str>)?,
        ],
    )
}

fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    let mut tray = TrayIconBuilder::with_id(TRAY).tooltip("Zoo Downloader").menu(&menu(app)?).show_menu_on_left_click(false);
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.on_menu_event(|app, event| match event.id.as_ref() {
        "show_hide" => toggle(app),
        "pause_all" | "resume_all" => {
            let (app, pause) = (app.clone(), event.id.as_ref() == "pause_all");
            tauri::async_runtime::spawn(async move {
                if let Some(d) = app.try_state::<crate::downloads::Downloads>() {
                    let _ = if pause { d.manager.pause_all().await } else { d.manager.resume_all().await };
                }
            });
        }
        "quit" => app.exit(0),
        _ => {}
    })
    .on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
            toggle(tray.app_handle());
        }
    })
    .build(app)?;
    Ok(())
}

/// The tray icon and the login item as `settings` say. `at_start`: the app is starting; a
/// development build then leaves the login item alone (it would point it at itself).
pub fn apply(app: &AppHandle, settings: &AppSettings, at_start: bool) {
    app.state::<CloseToTray>().0.store(settings.enable_system_tray, Ordering::SeqCst);
    match (settings.enable_system_tray, app.tray_by_id(TRAY)) {
        (true, None) => {
            if let Err(e) = create_tray(app) {
                log::warn!("cannot create the tray icon: {e}");
            }
        }
        (false, Some(_)) => {
            app.remove_tray_by_id(TRAY);
        }
        _ => {}
    }

    if at_start && cfg!(debug_assertions) {
        return;
    }
    // Enabled again at every start: the login item names this executable, which an update or a
    // move may have changed.
    let autostart = app.autolaunch();
    let result = if settings.launch_at_startup {
        autostart.enable()
    } else if autostart.is_enabled().unwrap_or(false) {
        autostart.disable()
    } else {
        Ok(())
    };
    if let Err(e) = result {
        log::warn!("cannot change starting at login: {e}");
    }
}

/// At startup: hidden in the tray when the login item started the app and the user wants
/// that (and has a tray icon to find it by); shown otherwise.
pub fn start(app: &AppHandle, settings: &AppSettings) {
    apply(app, settings, true);
    let at_login = std::env::args().any(|a| a == AUTOSTART_ARG);
    if at_login && settings.start_in_tray && settings.enable_system_tray {
        hide(app);
    } else {
        show(app);
    }
}

/// Closing the main window hides it in the tray while the tray icon is on; quitting is then
/// the tray menu's job.
pub fn on_window_event(window: &Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        let app = window.app_handle();
        if window.label() == MAIN_WINDOW && app.state::<CloseToTray>().0.load(Ordering::SeqCst) {
            api.prevent_close();
            hide(app);
        }
    }
}

/// The tray menu in the page's language.
#[tauri::command]
pub async fn app_set_tray_labels(labels: TrayLabels, app: AppHandle) -> CmdResult<()> {
    *app.state::<Labels>().0.lock().unwrap() = labels;
    if let Some(tray) = app.tray_by_id(TRAY) {
        let menu = menu(&app).map_err(|e| CommandError::new("tray", e.to_string()))?;
        tray.set_menu(Some(menu)).map_err(|e| CommandError::new("tray", e.to_string()))?;
    }
    Ok(())
}
