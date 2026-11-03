use std::time::Duration;
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde_json::Value;
use crate::error::AppError;

pub trait TranscriptionProvider: Send + Sync {
    fn provider_name(&self) -> &'static str;
    async fn transcribe(
        &self,
        audio_wav: &[u8],
        model: &str,
        language: &str,
        api_key: &str,
    ) -> Result<String, AppError>;
    async fn test_connection(&self, api_key: &str) -> Result<(), AppError>;
}

pub struct GroqWhisperProvider {
    client: Client,
}

impl GroqWhisperProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(35))
                .build()
                .unwrap_or_default(),
        }
    }
}

impl TranscriptionProvider for GroqWhisperProvider {
    fn provider_name(&self) -> &'static str {
        "groq"
    }

    async fn transcribe(
        &self,
        audio_wav: &[u8],
        model: &str,
        language: &str,
        api_key: &str,
    ) -> Result<String, AppError> {
        let file_part = Part::bytes(audio_wav.to_vec())
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| AppError::Transcription(format!("MIME creation failed: {}", e)))?;

        let mut form = Form::new()
            .part("file", file_part)
            .text("model", model.to_string())
            .text("response_format", "json");

        if language != "auto" && !language.trim().is_empty() {
            form = form.text("language", language.to_string());
        }

        let res = self
            .client
            .post("https://api.groq.com/openai/v1/audio/transcriptions")
            .bearer_auth(api_key.trim())
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AppError::Transcription("Groq Whisper API request timed out (35s)".to_string())
                } else {
                    AppError::Network(format!("Groq API connection error: {}", e))
                }
            })?;

        let status = res.status();
        let body = res.text().await.unwrap_or_default();

        if !status.is_success() {
            if status.as_u16() == 401 {
                return Err(AppError::Transcription(
                    "Invalid Groq API key (Authentication failed)".to_string(),
                ));
            } else if status.as_u16() == 429 {
                return Err(AppError::Transcription(
                    "Groq API rate limit reached. Please wait a moment.".to_string(),
                ));
            } else {
                return Err(AppError::Transcription(format!(
                    "Groq transcription failed ({}): {}",
                    status, body
                )));
            }
        }

        let json: Value = serde_json::from_str(&body)
            .map_err(|e| AppError::Json(format!("Failed to parse Groq response: {}", e)))?;

        let text = json["text"]
            .as_str()
            .unwrap_or("")
            .trim()
            .to_string();

        Ok(text)
    }

    async fn test_connection(&self, api_key: &str) -> Result<(), AppError> {
        let res = self
            .client
            .get("https://api.groq.com/openai/v1/models")
            .bearer_auth(api_key.trim())
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Groq test connection failed: {}", e)))?;

        if res.status().is_success() {
            Ok(())
        } else if res.status().as_u16() == 401 {
            Err(AppError::Transcription("Invalid Groq API key".to_string()))
        } else {
            Err(AppError::Transcription(format!(
                "Groq API responded with status {}",
                res.status()
            )))
        }
    }
}

pub struct OpenAIWhisperProvider {
    client: Client,
}

impl OpenAIWhisperProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(45))
                .build()
                .unwrap_or_default(),
        }
    }
}

impl TranscriptionProvider for OpenAIWhisperProvider {
    fn provider_name(&self) -> &'static str {
        "openai"
    }

    async fn transcribe(
        &self,
        audio_wav: &[u8],
        model: &str,
        language: &str,
        api_key: &str,
    ) -> Result<String, AppError> {
        let file_part = Part::bytes(audio_wav.to_vec())
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| AppError::Transcription(format!("MIME creation failed: {}", e)))?;

        let mut form = Form::new()
            .part("file", file_part)
            .text("model", model.to_string())
            .text("response_format", "json");

        if language != "auto" && !language.trim().is_empty() {
            form = form.text("language", language.to_string());
        }

        let res = self
            .client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .bearer_auth(api_key.trim())
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AppError::Transcription("OpenAI Whisper API request timed out (45s)".to_string())
                } else {
                    AppError::Network(format!("OpenAI API connection error: {}", e))
                }
            })?;

        let status = res.status();
        let body = res.text().await.unwrap_or_default();

        if !status.is_success() {
            if status.as_u16() == 401 {
                return Err(AppError::Transcription(
                    "Invalid OpenAI API key (Authentication failed)".to_string(),
                ));
            } else if status.as_u16() == 429 {
                return Err(AppError::Transcription(
                    "OpenAI API rate limit or quota exceeded.".to_string(),
                ));
            } else {
                return Err(AppError::Transcription(format!(
                    "OpenAI transcription failed ({}): {}",
                    status, body
                )));
            }
        }

        let json: Value = serde_json::from_str(&body)
            .map_err(|e| AppError::Json(format!("Failed to parse OpenAI response: {}", e)))?;

        let text = json["text"]
            .as_str()
            .unwrap_or("")
            .trim()
            .to_string();

        Ok(text)
    }

    async fn test_connection(&self, api_key: &str) -> Result<(), AppError> {
        let res = self
            .client
            .get("https://api.openai.com/v1/models")
            .bearer_auth(api_key.trim())
            .send()
            .await
            .map_err(|e| AppError::Network(format!("OpenAI test connection failed: {}", e)))?;

        if res.status().is_success() {
            Ok(())
        } else if res.status().as_u16() == 401 {
            Err(AppError::Transcription("Invalid OpenAI API key".to_string()))
        } else {
            Err(AppError::Transcription(format!(
                "OpenAI API responded with status {}",
                res.status()
            )))
        }
    }
}

pub async fn transcribe_audio(
    provider: &str,
    audio_wav: &[u8],
    model: &str,
    language: &str,
    api_key: &str,
) -> Result<String, AppError> {
    match provider.to_lowercase().as_str() {
        "groq" => {
            let groq = GroqWhisperProvider::new();
            groq.transcribe(audio_wav, model, language, api_key).await
        }
        "openai" => {
            let openai = OpenAIWhisperProvider::new();
            openai.transcribe(audio_wav, model, language, api_key).await
        }
        _ => Err(AppError::Transcription(format!(
            "Unsupported transcription provider: {}",
            provider
        ))),
    }
}

pub async fn test_transcription_api(provider: &str, api_key: &str) -> Result<(), AppError> {
    match provider.to_lowercase().as_str() {
        "groq" => GroqWhisperProvider::new().test_connection(api_key).await,
        "openai" => OpenAIWhisperProvider::new().test_connection(api_key).await,
        _ => Err(AppError::Transcription(format!(
            "Unknown provider: {}",
            provider
        ))),
    }
}

