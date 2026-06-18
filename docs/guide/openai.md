# OpenAI Provider

Create a provider with default settings. By default, the provider reads `OPENAI_API_KEY` and uses `https://api.openai.com/v1`.

```rust
use ai_provider_openai::{OpenAIProvider, OpenAIProviderSettings};

let provider = OpenAIProvider::new(OpenAIProviderSettings::default())?;
```

## Configure Settings

```rust
use ai_provider::Headers;
use ai_provider_openai::{OpenAIProvider, OpenAIProviderSettings};

let mut headers = Headers::new();
headers.insert("x-app".to_string(), "demo".to_string());

let provider = OpenAIProvider::new(OpenAIProviderSettings {
    api_key: Some("sk-...".to_string()),
    base_url: Some("https://api.openai.com/v1".to_string()),
    headers,
    client: reqwest::Client::new(),
})?;
```

## Language Models

Use `responses` for the OpenAI Responses API.

```rust
let model = provider.responses("gpt-4o-mini");
```

Use `chat` for chat completions.

```rust
let model = provider.chat("gpt-4o-mini");
```

## Other Model Types

```rust
let embedding = provider.embedding("text-embedding-3-small");
let image = provider.image("gpt-image-1");
let speech = provider.speech("gpt-4o-mini-tts");
let transcription = provider.transcription("gpt-4o-mini-transcribe");
```

## Environment Variables

| Variable | Description |
| --- | --- |
| `OPENAI_API_KEY` | API key used when `OpenAIProviderSettings::api_key` is not set. |
| `OPENAI_BASE_URL` | Optional base URL override used when `base_url` is not set. |
