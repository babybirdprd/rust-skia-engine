use async_trait::async_trait;
use serde_json::json;
use crate::{TtsProvider, TtsRequest, TtsResult, TtsError};
use base64::{Engine as _, engine::general_purpose};

pub struct GeminiProvider {
    api_key: String,
    client: reqwest::Client,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl TtsProvider for GeminiProvider {
    fn name(&self) -> &'static str {
        "gemini"
    }

    async fn synthesize(&self, request: TtsRequest) -> TtsResult<Vec<u8>> {
        let model = request.options.get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("gemini-2.5-flash-preview-tts");

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, self.api_key
        );

        // Gemini TTS config
        let body = json!({
            "contents": [{
                "parts": [{
                    "text": request.text
                }]
            }],
            "generationConfig": {
                "responseModalities": ["AUDIO"],
                "speechConfig": {
                    "voiceConfig": {
                        "prebuiltVoiceConfig": {
                            "voiceName": request.voice
                        }
                    }
                }
            }
        });

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(TtsError::ProviderError(format!("Gemini API error: {}", error_text)));
        }

        let response_json: serde_json::Value = response.json().await?;

        // Navigate the JSON to find the audio data
        // candidates[0].content.parts[0].inlineData.data
        let encoded_audio = response_json
            .get("candidates").and_then(|c| c.get(0))
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("inlineData"))
            .and_then(|d| d.get("data"))
            .and_then(|v| v.as_str());

        if let Some(base64_string) = encoded_audio {
            let audio_bytes = general_purpose::STANDARD
                .decode(base64_string)
                .map_err(|e| TtsError::ProviderError(format!("Failed to decode base64 audio: {}", e)))?;
            Ok(audio_bytes)
        } else {
             Err(TtsError::ProviderError("No audio data found in Gemini response".to_string()))
        }
    }
}
