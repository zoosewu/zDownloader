// System tray implementation
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime,
};

/// Create system tray icon with menu
pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let show_hide = MenuItem::with_id(app, "show_hide", "Show/Hide Window", true, None::<&str>)?;
    let pause_all = MenuItem::with_id(app, "pause_all", "Pause All", true, None::<&str>)?;
    let resume_all = MenuItem::with_id(app, "resume_all", "Resume All", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_hide, &pause_all, &resume_all, &quit])?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show_hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
            "pause_all" => {
                // Emit pause all event
                let _ = app.emit("tray-pause-all", ());
            }
            "resume_all" => {
                // Emit resume all event
                let _ = app.emit("tray-resume-all", ());
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// Update tray icon tooltip with download progress
#[allow(dead_code)]
pub fn update_tray_tooltip<R: Runtime>(
    app: &AppHandle<R>,
    active_downloads: usize,
    total_progress: f64,
) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id("main") {
        let tooltip = if active_downloads > 0 {
            format!(
                "Zoo Downloader - {} active ({:.1}%)",
                active_downloads, total_progress
            )
        } else {
            "Zoo Downloader - No active downloads".to_string()
        };
        tray.set_tooltip(Some(tooltip))?;
    }
    Ok(())
}
