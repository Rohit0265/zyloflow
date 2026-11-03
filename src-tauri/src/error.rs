use serde::Serialize;
use thiserror_impl::ThisError;

#[derive(Debug, ThisError)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("Network/API request error: {0}")]
    Network(String),

    #[error("JSON serialization error: {0}")]
    Json(String),

    #[error("Secure storage error: {0}")]
    SecureStorage(String),

    #[error("Audio recording error: {0}")]
    Audio(String),

    #[error("Transcription error: {0}")]
    Transcription(String),

    #[error("Refinement error: {0}")]
    Refinement(String),

    #[error("Text injection error: {0}")]
    TextInjection(String),

    #[error("Hotkey error: {0}")]
    Hotkey(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("{0}")]
    Other(String),
}

// Custom simple error implementation to avoid heavy proc macro dependencies if not needed
mod thiserror_impl {
    pub use std::fmt::Display;

    #[macro_export]
    macro_rules! impl_display {
        () => {};
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "I/O error: {}", e),
            AppError::Network(e) => write!(f, "Network/API error: {}", e),
            AppError::Json(e) => write!(f, "JSON error: {}", e),
            AppError::SecureStorage(e) => write!(f, "Keyring error: {}", e),
            AppError::Audio(e) => write!(f, "Audio error: {}", e),
            AppError::Transcription(e) => write!(f, "Transcription error: {}", e),
            AppError::Refinement(e) => write!(f, "Refinement error: {}", e),
            AppError::TextInjection(e) => write!(f, "Text injection error: {}", e),
            AppError::Hotkey(e) => write!(f, "Hotkey error: {}", e),
            AppError::Validation(e) => write!(f, "Validation error: {}", e),
            AppError::Other(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Json(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Network(err.to_string())
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
