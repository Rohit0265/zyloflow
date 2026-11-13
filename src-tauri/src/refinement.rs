use std::time::Duration;
use reqwest::Client;
use serde_json::{json, Value};
use crate::error::AppError;

#[allow(async_fn_in_trait)]
pub trait RefinementProvider: Send + Sync {
    fn provider_name(&self) -> &'static str;
    async fn refine(
        &self,
        raw_text: &str,
        model: &str,
        mode: &str,
        api_key: &str,
    ) -> Result<String, AppError>;
    async fn test_connection(&self, api_key: &str) -> Result<(), AppError>;
}

fn build_system_prompt(mode: &str) -> String {
    let mode_instruction = match mode.to_lowercase().as_str() {
        "minimal" => "\nRefinement style: Minimal correction. Only fix obvious spelling errors and basic punctuation. Retain exact wording and informal speech patterns.",
        "professional" => "\nRefinement style: Professional correction. Polish grammar, flow, and sentence structure for business or academic documents, emails, and technical correspondence.",
        _ => "\nRefinement style: Standard correction. Clean grammar, fix punctuation, and remove redundant vocal fillers naturally without changing the tone.",
    };

    format!(
        "You are an intelligent voice transcription refinement engine.\n\n\
        Your task is to clean up raw speech transcription.\n\n\
        Rules:\n\
        - Correct spelling and grammatical errors.\n\
        - Add appropriate punctuation.\n\
        - Remove unnecessary filler words when appropriate.\n\
        - Improve sentence structure without changing meaning.\n\
        - Preserve technical terms, code, names, and product names.\n\
        - Preserve the original language.\n\
        - Do not translate unless explicitly requested.\n\
        - Do not add new information.\n\
        - Do not answer questions contained in the transcription.\n\
        - Do not summarize.\n\
        - Do not explain your corrections.\n\
        - Return only the refined text.{}",
        mode_instruction
    )
}

// ==================== Gemini Provider ====================
pub struct GeminiProvider {
    client: Client,
}

impl GeminiProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }
}

impl RefinementProvider for GeminiProvider {
    fn provider_name(&self) -> &'static str {
        "gemini"
    }

    async fn refine(
        &self,
        raw_text: &str,
        model: &str,
        mode: &str,
        api_key: &str,
    ) -> Result<String, AppError> {
        let system_instruction = build_system_prompt(mode);
        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model,
            api_key.trim()
        );

        let body = json!({
            "system_instruction": {
                "parts": [{ "text": system_instruction }]
            },
            "contents": [
                {
                    "role": "user",
                    "parts": [{ "text": raw_text }]
                }
            ],
            "generationConfig": {
                "temperature": 0.2,
                "maxOutputTokens": 2048
            }
        });

        let res = self
            .client
            .post(&endpoint)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Gemini API error: {}", e)))?;

        let status = res.status();
        let resp_text = res.text().await.unwrap_or_default();

        if !status.is_success() {
            if status.as_u16() == 400 || status.as_u16() == 403 {
                return Err(AppError::Refinement(
                    "Invalid Google Gemini API key or model unavailable".to_string(),
                ));
            } else if status.as_u16() == 429 {
                return Err(AppError::Refinement(
                    "Gemini API rate limit reached".to_string(),
                ));
            } else {
                return Err(AppError::Refinement(format!(
                    "Gemini refinement failed ({}): {}",
                    status, resp_text
                )));
            }
        }

        let val: Value = serde_json::from_str(&resp_text)?;
        let refined = val["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or(raw_text)
            .trim()
            .to_string();

        Ok(refined)
    }

    async fn test_connection(&self, api_key: &str) -> Result<(), AppError> {
        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models?key={}",
            api_key.trim()
        );

        let res = self
            .client
            .get(&endpoint)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Gemini test connection failed: {}", e)))?;

        if res.status().is_success() {
            Ok(())
        } else if res.status().as_u16() == 400 || res.status().as_u16() == 403 {
            Err(AppError::Refinement("Invalid Gemini API key".to_string()))
        } else {
            Err(AppError::Refinement(format!(
                "Gemini API returned status {}",
                res.status()
            )))
        }
    }
}

// ==================== OpenAI-Compatible Provider (Groq, xAI, OpenAI) ====================
pub struct OpenAiCompatibleProvider {
    client: Client,
    base_url: String,
    provider_id: &'static str,
}

impl OpenAiCompatibleProvider {
    pub fn new(base_url: &str, provider_id: &'static str) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            base_url: base_url.to_string(),
            provider_id,
        }
    }
}

impl RefinementProvider for OpenAiCompatibleProvider {
    fn provider_name(&self) -> &'static str {
        self.provider_id
    }

    async fn refine(
        &self,
        raw_text: &str,
        model: &str,
        mode: &str,
        api_key: &str,
    ) -> Result<String, AppError> {
        let system_prompt = build_system_prompt(mode);
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let body = json!({
            "model": model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": raw_text }
            ],
            "temperature": 0.2
        });

        let res = self
            .client
            .post(&url)
            .bearer_auth(api_key.trim())
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("{} API error: {}", self.provider_id, e)))?;

        let status = res.status();
        let resp_text = res.text().await.unwrap_or_default();

        if !status.is_success() {
            if status.as_u16() == 401 {
                return Err(AppError::Refinement(format!(
                    "Invalid {} API key (Authentication failed)",
                    self.provider_id.to_uppercase()
                )));
            } else if status.as_u16() == 429 {
                return Err(AppError::Refinement(format!(
                    "{} API rate limit reached",
                    self.provider_id.to_uppercase()
                )));
            } else {
                return Err(AppError::Refinement(format!(
                    "{} refinement failed ({}): {}",
                    self.provider_id, status, resp_text
                )));
            }
        }

        let val: Value = serde_json::from_str(&resp_text)?;
        let refined = val["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or(raw_text)
            .trim()
            .to_string();

        Ok(refined)
    }

    async fn test_connection(&self, api_key: &str) -> Result<(), AppError> {
        let url = format!("{}/models", self.base_url.trim_end_matches('/'));
        let res = self
            .client
            .get(&url)
            .bearer_auth(api_key.trim())
            .send()
            .await
            .map_err(|e| AppError::Network(format!("{} test failed: {}", self.provider_id, e)))?;

        if res.status().is_success() {
            Ok(())
        } else if res.status().as_u16() == 401 {
            Err(AppError::Refinement(format!(
                "Invalid {} API key",
                self.provider_id.to_uppercase()
            )))
        } else {
            Err(AppError::Refinement(format!(
                "{} API returned status {}",
                self.provider_id,
                res.status()
            )))
        }
    }
}

pub async fn refine_transcription(
    provider: &str,
    raw_text: &str,
    model: &str,
    mode: &str,
    api_key: &str,
) -> Result<String, AppError> {
    match provider.to_lowercase().as_str() {
        "gemini" => {
            let p = GeminiProvider::new();
            p.refine(raw_text, model, mode, api_key).await
        }
        "groq" => {
            let p = OpenAiCompatibleProvider::new("https://api.groq.com/openai/v1", "groq");
            p.refine(raw_text, model, mode, api_key).await
        }
        "xai" => {
            let p = OpenAiCompatibleProvider::new("https://api.x.ai/v1", "xai");
            p.refine(raw_text, model, mode, api_key).await
        }
        "openai" => {
            let p = OpenAiCompatibleProvider::new("https://api.openai.com/v1", "openai");
            p.refine(raw_text, model, mode, api_key).await
        }
        _ => Err(AppError::Refinement(format!(
            "Unsupported refinement provider: {}",
            provider
        ))),
    }
}

pub async fn test_refinement_api(provider: &str, api_key: &str) -> Result<(), AppError> {
    match provider.to_lowercase().as_str() {
        "gemini" => GeminiProvider::new().test_connection(api_key).await,
        "groq" => {
            OpenAiCompatibleProvider::new("https://api.groq.com/openai/v1", "groq")
                .test_connection(api_key)
                .await
        }
        "xai" => {
            OpenAiCompatibleProvider::new("https://api.x.ai/v1", "xai")
                .test_connection(api_key)
                .await
        }
        "openai" => {
            OpenAiCompatibleProvider::new("https://api.openai.com/v1", "openai")
                .test_connection(api_key)
                .await
        }
        _ => Err(AppError::Refinement(format!("Unknown provider: {}", provider))),
    }
}

