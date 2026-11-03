pub mod audio;
pub mod error;
pub mod hotkeys;
pub mod lifecycle;
pub mod popup;
pub mod refinement;
pub mod secure_storage;
pub mod settings;
pub mod text_injection;
pub mod tray;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State, WindowEvent};
use error::AppError;
use lifecycle::AppState;
use secure_storage::SecureStorage;
use settings::{AppSettings, SettingsManager, TranscriptionRecord};
use text_injection::DisplayServer;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderValidationResult {
    pub provider: String,
    pub valid: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub is_active: bool,
    pub display_server: String,
    pub microphone_ready: bool,
    pub current_dictation_state: String,
}

#[derive(Debug, Deserialize)]
pub struct HotkeyConfig {
    pub toggle: String,
    pub start_app: String,
    pub stop_app: String,
}

#[derive(Debug, Serialize)]
pub struct HotkeyRegistrationResult {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MicPermissionResult {
    pub granted: bool,
    pub error: Option<String>,
}

// ==================== Tauri Commands ====================

#[tauri::command]
pub fn get_settings() -> AppSettings {
    SettingsManager::load_settings()
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<(), AppError> {
    SettingsManager::save_settings(&settings)
}

#[tauri::command]
pub fn get_api_key_configured(provider: String) -> bool {
    SecureStorage::has_key(&provider)
}

#[tauri::command]
pub fn save_api_key(provider: String, key: String) -> Result<(), AppError> {
    SecureStorage::save_key(&provider, &key)
}

#[tauri::command]
pub fn delete_api_key(provider: String) -> Result<(), AppError> {
    SecureStorage::delete_key(&provider)
}

#[tauri::command]
pub async fn test_api_key(
    provider: String,
    key: Option<String>,
) -> Result<ProviderValidationResult, AppError> {
    let key_to_use = match key {
        Some(k) if !k.trim().is_empty() => k,
        _ => SecureStorage::get_key(&provider)?,
    };

    let p = provider.to_lowercase();
    let res = if p == "groq" || p == "openai" {
        // Test STT or LLM
        match transcription::test_transcription_api(&p, &key_to_use).await {
            Ok(_) => Ok(()),
            Err(e) => refinement::test_refinement_api(&p, &key_to_use).await.map_err(|_| e),
        }
    } else {
        refinement::test_refinement_api(&p, &key_to_use).await
    };

    match res {
        Ok(_) => Ok(ProviderValidationResult {
            provider,
            valid: true,
            message: "Connected successfully".to_string(),
        }),
        Err(e) => Ok(ProviderValidationResult {
            provider,
            valid: false,
            message: e.to_string(),
        }),
    }
}

#[tauri::command]
pub fn get_system_status(state: State<'_, AppState>) -> SystemStatus {
    let mic_ready = audio::AudioRecorder::check_device_ready().unwrap_or(false);
    let display = DisplayServer::detect();

    SystemStatus {
        is_active: state.is_app_active(),
        display_server: display.as_str().to_string(),
        microphone_ready: mic_ready,
        current_dictation_state: "idle".to_string(),
    }
}

#[tauri::command]
pub async fn start_dictation(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    state.start_dictation(&app).await
}

#[tauri::command]
pub async fn stop_dictation(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    state.stop_dictation(&app).await
}

#[tauri::command]
pub async fn cancel_dictation(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    state.cancel_dictation(&app).await;
    Ok(())
}

#[tauri::command]
pub fn set_app_active(app: AppHandle, state: State<'_, AppState>, active: bool) -> Result<(), AppError> {
    state.set_active(&app, active);
    Ok(())
}

#[tauri::command]
pub fn get_history() -> Vec<TranscriptionRecord> {
    SettingsManager::load_history()
}

#[tauri::command]
pub fn delete_history_item(id: String) -> Result<(), AppError> {
    SettingsManager::delete_history_record(&id)
}

#[tauri::command]
pub fn clear_history() -> Result<(), AppError> {
    SettingsManager::clear_history()
}

#[tauri::command]
pub fn register_hotkeys(app: AppHandle, shortcuts: HotkeyConfig) -> HotkeyRegistrationResult {
    let mut settings = SettingsManager::load_settings();
    settings.hotkey_toggle = shortcuts.toggle;
    settings.hotkey_start_app = shortcuts.start_app;
    settings.hotkey_stop_app = shortcuts.stop_app;

    if let Err(e) = SettingsManager::save_settings(&settings) {
        return HotkeyRegistrationResult {
            success: false,
            error: Some(e.to_string()),
        };
    }

    match hotkeys::HotkeyManager::register_all(&app) {
        Ok(_) => HotkeyRegistrationResult {
            success: true,
            error: None,
        },
        Err(e) => HotkeyRegistrationResult {
            success: false,
            error: Some(e.to_string()),
        },
    }
}

#[tauri::command]
pub fn check_microphone_permission() -> MicPermissionResult {
    match audio::AudioRecorder::check_device_ready() {
        Ok(_) => MicPermissionResult {
            granted: true,
            error: None,
        },
        Err(e) => MicPermissionResult {
            granted: false,
            error: Some(e.to_string()),
        },
    }
}

#[tauri::command]
pub fn complete_onboarding() -> Result<(), AppError> {
    let mut settings = SettingsManager::load_settings();
    settings.is_first_launch = false;
    SettingsManager::save_settings(&settings)
}

#[tauri::command]
pub fn hide_main_window(app: AppHandle) -> Result<(), AppError> {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
    Ok(())
}

#[tauri::command]
pub fn hide_popup_window(app: AppHandle) -> Result<(), AppError> {
    popup::PopupManager::hide_popup(&app);
    Ok(())
}

// ==================== Application Run Setup ====================

pub fn run() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .manage(app_state)
        .setup(|app| {
            let handle = app.handle().clone();

            // Setup System Tray
            if let Err(e) = tray::create_tray(&handle) {
                log::error!("Failed to create system tray: {}", e);
            }

            // Register initial global hotkeys
            if let Err(e) = hotkeys::HotkeyManager::register_all(&handle) {
                log::warn!("Global hotkey registration notice: {}", e);
            }

            // Check if launched with --minimized or settings has start_minimized
            let settings = SettingsManager::load_settings();
            let args: Vec<String> = std::env::args().collect();
            let launched_minimized = args.contains(&"--minimized".to_string()) || settings.start_minimized;

            if launched_minimized {
                if let Some(main_win) = app.get_webview_window("main") {
                    let _ = main_win.hide();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // If closing main window and background mode is enabled, hide instead of exiting
                if window.label() == "main" {
                    let settings = SettingsManager::load_settings();
                    if settings.background_mode {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            get_api_key_configured,
            save_api_key,
            delete_api_key,
            test_api_key,
            get_system_status,
            start_dictation,
            stop_dictation,
            cancel_dictation,
            set_app_active,
            get_history,
            delete_history_item,
            clear_history,
            register_hotkeys,
            check_microphone_permission,
            complete_onboarding,
            hide_main_window,
            hide_popup_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ZyloFlow application");
}
