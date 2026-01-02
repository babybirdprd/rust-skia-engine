pub mod openai;
pub mod elevenlabs;
pub mod gemini;

use crate::TtsProvider;

pub enum ProviderType {
    OpenAI,
    ElevenLabs,
    Gemini,
}

pub fn create_provider(provider_type: ProviderType, api_key: String) -> Box<dyn TtsProvider> {
    match provider_type {
        ProviderType::OpenAI => Box::new(openai::OpenAIProvider::new(api_key)),
        ProviderType::ElevenLabs => Box::new(elevenlabs::ElevenLabsProvider::new(api_key)),
        ProviderType::Gemini => Box::new(gemini::GeminiProvider::new(api_key)),
    }
}
