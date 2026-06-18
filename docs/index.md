---
layout: home

hero:
  name: AI Provider SDK
  text: Rust provider abstractions for AI applications
  tagline: A compact workspace for building provider adapters, shared request utilities, and provider-neutral AI function entry points.
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: View Architecture
      link: /guide/architecture

features:
  - title: Provider-first traits
    details: Define language, embedding, image, speech, transcription, and reranking behavior behind small async traits.
  - title: Core function layer
    details: Call generate_text, stream_text, embed, generate_image, generate_speech, and transcribe without binding app code to one provider.
  - title: OpenAI adapter
    details: Includes an OpenAI implementation for text generation, streaming, embeddings, images, speech, and transcription.
  - title: HTTP utilities
    details: Shared API helpers handle headers, authentication, JSON, multipart requests, bytes responses, and SSE parsing.
---

## Workspace Packages

| Package | Purpose |
| --- | --- |
| `ai-provider` | Provider traits, result types, model call options, stream events, warnings, and errors. |
| `ai-provider-utils` | Shared request helpers for provider implementations. |
| `ai-core` | Provider-neutral functions that call model trait implementations. |
| `ai-provider-openai` | OpenAI provider implementation and examples. |

## Quick Example

```rust
use ai_core::generate_text;
use ai_provider::{LanguageMessage, LanguageModelCallOptions};
use ai_provider_openai::{OpenAIProvider, OpenAIProviderSettings};

#[tokio::main]
async fn main() -> Result<(), ai_provider::AiError> {
    let provider = OpenAIProvider::new(OpenAIProviderSettings::default())?;
    let model = provider.responses("gpt-4o-mini");

    let result = generate_text(
        &model,
        LanguageModelCallOptions {
            prompt: vec![LanguageMessage::user("Write a haiku about Rust traits.")],
            max_output_tokens: Some(80),
            ..Default::default()
        },
    )
    .await?;

    println!("{result:#?}");
    Ok(())
}
```
