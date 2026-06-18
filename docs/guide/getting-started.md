# Getting Started

This workspace is organized around provider-neutral traits and thin provider adapters. Application code calls the functions in `ai-core`, while provider packages implement the model traits from `ai-provider`.

## Requirements

- Rust 1.95 or newer
- An OpenAI API key for OpenAI examples

## Build

```bash
cargo build
```

## Test

```bash
cargo test
```

## Run an OpenAI Example

Set `OPENAI_API_KEY`, then run one of the provider examples.

```bash
export OPENAI_API_KEY=sk-...
cargo run -p ai-provider-openai --example generate_text_openai
```

## Use a Model

```rust
use ai_core::generate_text;
use ai_provider::{LanguageMessage, LanguageModelCallOptions};
use ai_provider_openai::{OpenAIProvider, OpenAIProviderSettings};

let provider = OpenAIProvider::new(OpenAIProviderSettings::default())?;
let model = provider.responses("gpt-4o-mini");

let result = generate_text(
    &model,
    LanguageModelCallOptions {
        prompt: vec![LanguageMessage::user("Summarize provider adapters.")],
        max_output_tokens: Some(120),
        ..Default::default()
    },
)
.await?;
```

## Available Examples

| Example | Command |
| --- | --- |
| Generate text | `cargo run -p ai-provider-openai --example generate_text_openai` |
| Stream text | `cargo run -p ai-provider-openai --example stream_text_openai` |
| Embeddings | `cargo run -p ai-provider-openai --example embed_openai` |
| Image generation | `cargo run -p ai-provider-openai --example generate_image_openai` |
| Speech | `cargo run -p ai-provider-openai --example speech_openai` |
| Transcription | `cargo run -p ai-provider-openai --example transcribe_openai` |
