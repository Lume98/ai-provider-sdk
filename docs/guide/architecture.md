# Architecture

The SDK follows a small adapter architecture:

```text
ai-core
   |
   v
ai-provider traits
   ^
   |
provider implementations, such as ai-provider-openai
   |
   v
ai-provider-utils
```

## `ai-provider`

`ai-provider` defines the contracts that every provider implementation follows.

- `LanguageModel`
- `EmbeddingModel`
- `ImageModel`
- `SpeechModel`
- `TranscriptionModel`
- `RerankingModel`
- `Provider`

It also owns common response types such as `GenerateResult`, `EmbeddingResult`, `ImageResult`, `SpeechResult`, `TranscriptionResult`, `Usage`, `Warning`, and `AiError`.

## `ai-core`

`ai-core` exposes provider-neutral functions:

- `generate_text`
- `stream_text`
- `embed`
- `generate_image`
- `generate_speech`
- `transcribe`
- `rerank`

Each function accepts a trait object, calls the matching model method, and returns a common result type.

## `ai-provider-utils`

`ai-provider-utils` contains reusable implementation helpers for provider packages:

- API key loading from explicit settings or environment variables
- Header merging
- JSON and multipart POST helpers
- Bytes response handling
- Server-sent event parsing
- Provider option parsing

## `ai-provider-openai`

`ai-provider-openai` implements the provider traits for OpenAI endpoints. It supports:

- Responses and chat completions for language models
- Streaming language responses
- Embeddings
- Image generation
- Speech generation
- Transcription

Unsupported provider capabilities return `AiError::Unsupported` instead of silently ignoring the call.
