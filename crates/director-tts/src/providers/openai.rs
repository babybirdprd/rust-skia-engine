use async_trait::async_trait;
use serde_json::json;
use crate::{TtsProvider, TtsRequest, TtsResult, TtsError};

pub struct OpenAIProvider {
    api_key: String,
    client: reqwest::Client,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl TtsProvider for OpenAIProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    async fn synthesize(&self, request: TtsRequest) -> TtsResult<Vec<u8>> {
        let model = request.options.get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("tts-1");

        let response = self.client
            .post("https://api.openai.com/v1/audio/speech")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "model": model,
                "input": request.text,
                "voice": request.voice,
                "speed": request.speed,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(TtsError::ProviderError(format!("OpenAI API error: {}", error_text)));
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}
