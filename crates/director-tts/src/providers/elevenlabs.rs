use async_trait::async_trait;
use serde_json::json;
use crate::{TtsProvider, TtsRequest, TtsResult, TtsError};

pub struct ElevenLabsProvider {
    api_key: String,
    client: reqwest::Client,
}

impl ElevenLabsProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl TtsProvider for ElevenLabsProvider {
    fn name(&self) -> &'static str {
        "elevenlabs"
    }

    async fn synthesize(&self, request: TtsRequest) -> TtsResult<Vec<u8>> {
        let model_id = request.options.get("model_id")
            .and_then(|v| v.as_str())
            .unwrap_or("eleven_monolingual_v1");

        // Voice ID is passed as the 'voice' field in the request
        let voice_id = &request.voice;

        let url = format!("https://api.elevenlabs.io/v1/text-to-speech/{}", voice_id);

        let mut body = json!({
            "text": request.text,
            "model_id": model_id,
        });

        if let Some(settings) = request.options.get("voice_settings") {
            body["voice_settings"] = settings.clone();
        }

        let response = self.client
            .post(&url)
            .header("xi-api-key", &self.api_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(TtsError::ProviderError(format!("ElevenLabs API error: {}", error_text)));
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}
