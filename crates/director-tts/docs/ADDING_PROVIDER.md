# Adding a New TTS Provider

To add a new Text-to-Speech (TTS) provider to `director-tts`, follow these steps:

1.  **Create a new module**: Create a new file in `crates/director-tts/src/providers/` (e.g., `myprovider.rs`).
2.  **Implement the `TtsProvider` trait**: Implement the `TtsProvider` trait for your provider struct.
3.  **Register the provider**: Add your module to `crates/director-tts/src/providers/mod.rs` and update the `create_provider` factory function.

## Step 1: Create Module

Create `crates/director-tts/src/providers/myprovider.rs`:

```rust
use async_trait::async_trait;
use crate::{TtsProvider, TtsRequest, TtsResult, TtsError};

pub struct MyProvider {
    api_key: String,
    client: reqwest::Client,
}

impl MyProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl TtsProvider for MyProvider {
    fn name(&self) -> &'static str {
        "myprovider"
    }

    async fn synthesize(&self, request: TtsRequest) -> TtsResult<Vec<u8>> {
        // Implement the API call here
        // return Ok(audio_bytes);
        todo!("Implement synthesize")
    }
}
```

## Step 2: Register Provider

Update `crates/director-tts/src/providers/mod.rs`:

```rust
pub mod myprovider; // Add this line

pub enum ProviderType {
    OpenAI,
    ElevenLabs,
    Gemini,
    MyProvider, // Add this variant
}

pub fn create_provider(provider_type: ProviderType, api_key: String) -> Box<dyn TtsProvider> {
    match provider_type {
        ProviderType::OpenAI => Box::new(openai::OpenAIProvider::new(api_key)),
        ProviderType::ElevenLabs => Box::new(elevenlabs::ElevenLabsProvider::new(api_key)),
        ProviderType::Gemini => Box::new(gemini::GeminiProvider::new(api_key)),
        ProviderType::MyProvider => Box::new(myprovider::MyProvider::new(api_key)), // Add this line
    }
}
```

## Tips

- Use `request.options` (a `serde_json::Value`) to pass provider-specific parameters like model IDs, stability settings, etc.
- Handle errors using `TtsError::ProviderError` to provide descriptive failure messages.
- Ensure your implementation is `Send + Sync`.
