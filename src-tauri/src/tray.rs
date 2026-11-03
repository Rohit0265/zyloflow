use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};
use crate::lifecycle::AppState;

pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    let open_i = MenuItem::with_id(app, "open", "Open ZyloFlow", true, None::<&str>)?;
    let start_dict_i = MenuItem::with_id(app, "start_dictation", "Start Dictation", true, None::<&str>)?;
    let pause_i = MenuItem::with_id(app, "pause", "Pause Application", true, None::<&str>)?;
    let resume_i = MenuItem::with_id(app, "resume", "Resume Application", true, None::<&str>)?;
    let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &open_i,
            &start_dict_i,
            &pause_i,
            &resume_i,
            &settings_i,
            &quit_i,
        ],
    )?;

    let tray = TrayIconBuilder::with_id("zyloflow-tray")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "open" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "start_dictation" => {
                    let app_clone = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let state = app_clone.state::<AppState>();
                        let _ = state.start_dictation(&app_clone).await;
                    });
                }
                "pause" => {
                    let state = app.state::<AppState>();
                    state.set_active(app, false);
                }
                "resume" => {
                    let state = app.state::<AppState>();
                    state.set_active(app, true);
                }
                "settings" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.eval("window.zyloflowNavigate && window.zyloflowNavigate('settings')");
                    }
                }
                "quit" => {
                    let state = app.state::<AppState>();
                    state.shutdown(app);
                    app.exit(0);
                }
                _ => {}
            }
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

    let _ = tray.set_tooltip(Some("ZyloFlow — AI Voice-to-Text"));

    Ok(())
}
