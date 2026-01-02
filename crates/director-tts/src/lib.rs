use async_trait::async_trait;
use thiserror::Error;

pub mod providers;

#[derive(Error, Debug)]
pub enum TtsError {
    #[error("API request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Provider error: {0}")]
    ProviderError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type TtsResult<T> = Result<T, TtsError>;

#[derive(Debug, Clone)]
pub struct TtsRequest {
    pub text: String,
    pub voice: String,
    pub speed: f32,
    /// Provider-specific configuration (e.g., model ID)
    pub options: serde_json::Value,
}

impl Default for TtsRequest {
    fn default() -> Self {
        Self {
            text: String::new(),
            voice: String::new(),
            speed: 1.0,
            options: serde_json::json!({}),
        }
    }
}

#[async_trait]
pub trait TtsProvider: Send + Sync {
    /// Synthesizes text to speech and returns the audio bytes (usually MP3 or WAV).
    async fn synthesize(&self, request: TtsRequest) -> TtsResult<Vec<u8>>;

    /// Returns the provider name (e.g., "openai", "elevenlabs").
    fn name(&self) -> &'static str;
}
