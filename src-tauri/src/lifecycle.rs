use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::audio::AudioRecorder;
use crate::error::AppError;
use crate::popup::PopupManager;
use crate::refinement::refine_transcription;
use crate::secure_storage::SecureStorage;
use crate::settings::{SettingsManager, TranscriptionRecord};
use crate::text_injection::TextInjector;
use crate::transcription::transcribe_audio;

pub struct AppState {
    pub is_active: Arc<AtomicBool>,
    pub audio_recorder: Arc<Mutex<AudioRecorder>>,
    pub is_processing: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            is_active: Arc::new(AtomicBool::new(true)),
            audio_recorder: Arc::new(Mutex::new(AudioRecorder::new())),
            is_processing: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_active(&self, app: &AppHandle, active: bool) {
        self.is_active.store(active, Ordering::SeqCst);
        let _ = app.emit("app-active-changed", serde_json::json!({ "is_active": active }));
        log::info!("Application active state set to: {}", active);
    }

    pub fn is_app_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    pub async fn start_dictation(&self, app: &AppHandle) -> Result<(), AppError> {
        if !self.is_app_active() {
            log::info!("Dictation triggered while application is paused. Ignoring.");
            return Ok(());
        }

        if self.is_processing.load(Ordering::SeqCst) {
            log::warn!("Dictation is already processing. Ignoring start request.");
            return Ok(());
        }

        let settings = SettingsManager::load_settings();

        // Check if transcription key exists
        let _stt_key = match SecureStorage::get_key(&settings.transcription_provider) {
            Ok(k) => k,
            Err(_) => {
                let err_msg = format!(
                    "Missing API key for STT provider '{}'. Please configure it in Settings.",
                    settings.transcription_provider
                );
                PopupManager::show_popup(app, "error", Some(&err_msg));
                let app_c = app.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    PopupManager::hide_popup(&app_c);
                });
                return Err(AppError::Validation(err_msg));
            }
        };

        // Check refinement key if enabled
        if settings.enable_refinement {
            if let Err(_) = SecureStorage::get_key(&settings.refinement_provider) {
                let err_msg = format!(
                    "Missing API key for refinement provider '{}'.",
                    settings.refinement_provider
                );
                PopupManager::show_popup(app, "error", Some(&err_msg));
                let app_c = app.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    PopupManager::hide_popup(&app_c);
                });
                return Err(AppError::Validation(err_msg));
            }
        }

        // Show popup in "listening" state
        PopupManager::show_popup(app, "listening", Some("Listening..."));

        // Start microphone recording
        let mut recorder = self.audio_recorder.lock().await;
        if let Err(e) = recorder.start_recording() {
            let msg = format!("Microphone error: {}", e);
            PopupManager::update_state(app, "error", None, None, Some(&msg));
            let app_c = app.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(3)).await;
                PopupManager::hide_popup(&app_c);
            });
            return Err(e);
        }

        log::info!("Microphone recording started.");
        Ok(())
    }

    pub async fn stop_dictation(&self, app: &AppHandle) -> Result<(), AppError> {
        let (wav_bytes, duration_secs) = {
            let mut recorder = self.audio_recorder.lock().await;
            if !recorder.is_recording() {
                return Ok(());
            }
            match recorder.stop_recording() {
                Ok(data) => data,
                Err(e) => {
                    log::warn!("Audio stop notice: {}", e);
                    PopupManager::hide_popup(app);
                    return Ok(());
                }
            }
        };

        self.is_processing.store(true, Ordering::SeqCst);
        let settings = SettingsManager::load_settings();

        // 1. Transcribe audio
        PopupManager::update_state(app, "transcribing", Some("Transcribing..."), None, None);

        let stt_key = SecureStorage::get_key(&settings.transcription_provider)?;
        let raw_transcript = match transcribe_audio(
            &settings.transcription_provider,
            &wav_bytes,
            &settings.transcription_model,
            &settings.audio_language,
            &stt_key,
        )
        .await
        {
            Ok(t) => t,
            Err(e) => {
                self.is_processing.store(false, Ordering::SeqCst);
                let err_str = e.to_string();
                PopupManager::update_state(app, "error", None, None, Some(&err_str));
                let app_c = app.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(4)).await;
                    PopupManager::hide_popup(&app_c);
                });
                return Err(e);
            }
        };

        if raw_transcript.trim().is_empty() {
            self.is_processing.store(false, Ordering::SeqCst);
            PopupManager::hide_popup(app);
            return Ok(());
        }

        // 2. AI Refinement (if enabled)
        let final_text = if settings.enable_refinement {
            PopupManager::update_state(app, "refining", Some("Refining text..."), None, None);
            let llm_key = SecureStorage::get_key(&settings.refinement_provider)?;

            match refine_transcription(
                &settings.refinement_provider,
                &raw_transcript,
                &settings.refinement_model,
                &settings.refinement_mode,
                &llm_key,
            )
            .await
            {
                Ok(refined) => refined,
                Err(e) => {
                    log::warn!("Refinement error, falling back to raw transcription: {}", e);
                    raw_transcript.clone()
                }
            }
        } else {
            raw_transcript.clone()
        };

        // 3. Insert text into currently focused external input
        PopupManager::update_state(app, "inserting", Some("Inserting text..."), None, None);
        if let Err(e) = TextInjector::inject_text(&final_text, &settings.clipboard_fallback_mode).await {
            log::error!("Text injection notice: {}", e);
        }

        // 4. Save history if enabled
        if settings.enable_history {
            let char_count = final_text.chars().count();
            let word_count = final_text.split_whitespace().count();
            let record = TranscriptionRecord {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now().timestamp_millis(),
                raw_text: raw_transcript,
                refined_text: final_text.clone(),
                duration_seconds: duration_secs,
                transcription_provider: settings.transcription_provider,
                refinement_provider: if settings.enable_refinement {
                    settings.refinement_provider
                } else {
                    String::new()
                },
                char_count,
                word_count,
            };
            let _ = SettingsManager::add_history_record(record);
        }

        // 5. Update state to completed
        PopupManager::update_state(
            app,
            "completed",
            Some("Text inserted"),
            Some(&final_text),
            None,
        );
        self.is_processing.store(false, Ordering::SeqCst);

        // Auto-dismiss popup after 1.5s
        let app_c = app.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(1500)).await;
            PopupManager::hide_popup(&app_c);
        });

        Ok(())
    }

    pub async fn cancel_dictation(&self, app: &AppHandle) {
        {
            let mut recorder = self.audio_recorder.lock().await;
            recorder.cancel_recording();
        }
        self.is_processing.store(false, Ordering::SeqCst);
        PopupManager::hide_popup(app);
    }

    pub fn shutdown(&self, app: &AppHandle) {
        let app_c = app.clone();
        tokio::spawn(async move {
            let state = app_c.state::<AppState>();
            let mut recorder = state.audio_recorder.lock().await;
            recorder.cancel_recording();
            PopupManager::hide_popup(&app_c);
        });
    }
}

