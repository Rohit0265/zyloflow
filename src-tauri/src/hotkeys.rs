use std::str::FromStr;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use crate::error::AppError;
use crate::lifecycle::AppState;
use crate::settings::SettingsManager;

pub struct HotkeyManager;

impl HotkeyManager {
    /// Registers or re-registers all 3 configured global hotkeys
    pub fn register_all(app: &AppHandle) -> Result<(), AppError> {
        let settings = SettingsManager::load_settings();

        // First unregister existing shortcuts to prevent conflicts
        let _ = app.global_shortcut().unregister_all();

        // 1. Toggle Dictation shortcut
        Self::register_single(app, &settings.hotkey_toggle, "toggle")?;

        // 2. Start Application shortcut
        Self::register_single(app, &settings.hotkey_start_app, "start")?;

        // 3. Stop Application shortcut
        Self::register_single(app, &settings.hotkey_stop_app, "stop")?;

        log::info!("All global shortcuts registered successfully.");
        Ok(())
    }

    fn register_single(app: &AppHandle, shortcut_str: &str, action: &'static str) -> Result<(), AppError> {
        let shortcut = match Shortcut::from_str(shortcut_str) {
            Ok(s) => s,
            Err(e) => {
                return Err(AppError::Hotkey(format!(
                    "Invalid shortcut format '{}': {}",
                    shortcut_str, e
                )));
            }
        };

        let app_handle = app.clone();
        app.global_shortcut()
            .on_shortcut(shortcut, move |_app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    let app_c = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let state = app_c.state::<AppState>();
                        match action {
                            "toggle" => {
                                let is_rec = {
                                    let recorder = state.audio_recorder.lock().await;
                                    recorder.is_recording()
                                };

                                if is_rec {
                                    let _ = state.stop_dictation(&app_c).await;
                                } else {
                                    let _ = state.start_dictation(&app_c).await;
                                }
                            }
                            "start" => {
                                state.set_active(&app_c, true);
                            }
                            "stop" => {
                                state.cancel_dictation(&app_c).await;
                                state.set_active(&app_c, false);
                            }
                            _ => {}
                        }
                    });
                }
            })
            .map_err(|e| {
                AppError::Hotkey(format!(
                    "Failed to register global hotkey '{}': {}. It may already be bound by the OS.",
                    shortcut_str, e
                ))
            })?;

        Ok(())
    }

    pub fn unregister_all(app: &AppHandle) {
        let _ = app.global_shortcut().unregister_all();
    }
}

