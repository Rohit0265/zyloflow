use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub is_first_launch: bool,
    pub transcription_provider: String,
    pub transcription_model: String,
    pub refinement_provider: String,
    pub refinement_model: String,
    pub refinement_mode: String,
    pub enable_refinement: bool,
    pub enable_history: bool,
    pub audio_language: String,
    pub hotkey_toggle: String,
    pub hotkey_start_app: String,
    pub hotkey_stop_app: String,
    pub launch_at_startup: bool,
    pub start_minimized: bool,
    pub background_mode: bool,
    pub clipboard_fallback_mode: String,
    pub enable_notifications: bool,
    pub microphone_device_id: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            is_first_launch: true,
            transcription_provider: "groq".to_string(),
            transcription_model: "whisper-large-v3-turbo".to_string(),
            refinement_provider: "gemini".to_string(),
            refinement_model: "gemini-2.5-flash".to_string(),
            refinement_mode: "standard".to_string(),
            enable_refinement: true,
            enable_history: true,
            audio_language: "auto".to_string(),
            hotkey_toggle: "Control+Alt+Space".to_string(),
            hotkey_start_app: "Control+Alt+S".to_string(),
            hotkey_stop_app: "Control+Alt+X".to_string(),
            launch_at_startup: false,
            start_minimized: false,
            background_mode: true,
            clipboard_fallback_mode: "auto".to_string(),
            enable_notifications: true,
            microphone_device_id: "default".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionRecord {
    pub id: String,
    pub timestamp: i64,
    pub raw_text: String,
    pub refined_text: String,
    pub duration_seconds: f64,
    pub transcription_provider: String,
    pub refinement_provider: String,
    pub char_count: usize,
    pub word_count: usize,
}

pub struct SettingsManager;

impl SettingsManager {
    pub fn get_app_dir() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            let path = config_dir.join("zyloflow");
            let _ = create_dir_all(&path);
            path
        } else {
            PathBuf::from("/tmp/zyloflow")
        }
    }

    fn settings_path() -> PathBuf {
        Self::get_app_dir().join("settings.json")
    }

    fn history_path() -> PathBuf {
        Self::get_app_dir().join("history.json")
    }

    pub fn load_settings() -> AppSettings {
        let path = Self::settings_path();
        if !path.exists() {
            let default_settings = AppSettings::default();
            let _ = Self::save_settings(&default_settings);
            return default_settings;
        }

        match File::open(&path) {
            Ok(mut file) => {
                let mut content = String::new();
                if file.read_to_string(&mut content).is_ok() {
                    serde_json::from_str(&content).unwrap_or_default()
                } else {
                    AppSettings::default()
                }
            }
            Err(_) => AppSettings::default(),
        }
    }

    pub fn save_settings(settings: &AppSettings) -> Result<(), AppError> {
        let path = Self::settings_path();
        let json = serde_json::to_string_pretty(settings)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn load_history() -> Vec<TranscriptionRecord> {
        let path = Self::history_path();
        if !path.exists() {
            return Vec::new();
        }

        match File::open(&path) {
            Ok(mut file) => {
                let mut content = String::new();
                if file.read_to_string(&mut content).is_ok() {
                    serde_json::from_str(&content).unwrap_or_default()
                } else {
                    Vec::new()
                }
            }
            Err(_) => Vec::new(),
        }
    }

    pub fn add_history_record(record: TranscriptionRecord) -> Result<(), AppError> {
        let mut history = Self::load_history();
        // Insert new items at top
        history.insert(0, record);
        // Keep last 100 entries max
        if history.len() > 100 {
            history.truncate(100);
        }

        let path = Self::history_path();
        let json = serde_json::to_string_pretty(&history)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn delete_history_record(id: &str) -> Result<(), AppError> {
        let mut history = Self::load_history();
        history.retain(|item| item.id != id);

        let path = Self::history_path();
        let json = serde_json::to_string_pretty(&history)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn clear_history() -> Result<(), AppError> {
        let path = Self::history_path();
        let json = serde_json::to_string_pretty(&Vec::<TranscriptionRecord>::new())?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}
