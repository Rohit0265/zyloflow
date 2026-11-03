use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow};

#[derive(Debug, Clone, Serialize)]
pub struct PopupStatePayload {
    pub state: String,
    pub message: Option<String>,
    pub inserted_text: Option<String>,
    pub error: Option<String>,
}

pub struct PopupManager;

impl PopupManager {
    pub fn get_popup_window(app: &AppHandle) -> Option<WebviewWindow> {
        app.get_webview_window("popup")
    }

    pub fn position_popup(window: &WebviewWindow) {
        if let Ok(Some(monitor)) = window.current_monitor() {
            let screen_size = monitor.size();
            let window_width = 360;
            let window_height = 72;

            // Position at bottom-center of the screen, 60px above bottom edge
            let x = (screen_size.width as i32 - window_width) / 2;
            let y = screen_size.height as i32 - window_height - 60;

            let _ = window.set_position(PhysicalPosition::new(x, y));
        }
    }

    pub fn show_popup(
        app: &AppHandle,
        state: &str,
        message: Option<&str>,
    ) {
        if let Some(window) = Self::get_popup_window(app) {
            Self::position_popup(&window);
            let _ = window.set_always_on_top(true);
            let _ = window.show();

            // Emit state update event
            let payload = PopupStatePayload {
                state: state.to_string(),
                message: message.map(|s| s.to_string()),
                inserted_text: None,
                error: None,
            };
            let _ = app.emit("dictation-state-changed", payload);
        }
    }

    pub fn update_state(
        app: &AppHandle,
        state: &str,
        message: Option<&str>,
        inserted_text: Option<&str>,
        error: Option<&str>,
    ) {
        let payload = PopupStatePayload {
            state: state.to_string(),
            message: message.map(|s| s.to_string()),
            inserted_text: inserted_text.map(|s| s.to_string()),
            error: error.map(|s| s.to_string()),
        };
        let _ = app.emit("dictation-state-changed", payload);
    }

    pub fn hide_popup(app: &AppHandle) {
        if let Some(window) = Self::get_popup_window(app) {
            let _ = window.hide();
        }
        let payload = PopupStatePayload {
            state: "idle".to_string(),
            message: None,
            inserted_text: None,
            error: None,
        };
        let _ = app.emit("dictation-state-changed", payload);
    }
}
