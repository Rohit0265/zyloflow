use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use crate::audio;
use crate::error::AppError;
use crate::hotkeys;
use crate::lifecycle::AppState;
use crate::popup;
use crate::refinement;
use crate::secure_storage::SecureStorage;
use crate::settings::{AppSettings, SettingsManager, TranscriptionRecord};
use crate::text_injection::DisplayServer;
use crate::transcription;

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
