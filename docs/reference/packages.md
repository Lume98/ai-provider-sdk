# Package Reference

## `ai-provider`

Core contracts and shared types.

| Item | Description |
| --- | --- |
| `AiError` | Common error enum for API, authentication, argument, parsing, unsupported, and missing model failures. |
| `Provider` | Factory trait for model interfaces. |
| `LanguageModel` | Async text generation and streaming contract. |
| `EmbeddingModel` | Async embedding contract. |
| `ImageModel` | Async image generation contract. |
| `SpeechModel` | Async speech generation contract. |
| `TranscriptionModel` | Async transcription contract. |
| `RerankingModel` | Async reranking contract. |

## `ai-core`

Provider-neutral call functions.

| Function | Model Trait |
| --- | --- |
| `generate_text` | `LanguageModel` |
| `stream_text` | `LanguageModel` |
| `embed` | `EmbeddingModel` |
| `generate_image` | `ImageModel` |
| `generate_speech` | `SpeechModel` |
| `transcribe` | `TranscriptionModel` |
| `rerank` | `RerankingModel` |

## `ai-provider-utils`

Shared helpers for provider implementations.

| Helper | Purpose |
| --- | --- |
| `load_api_key` | Resolve an explicit API key or read one from the environment. |
| `merge_headers` | Merge default provider headers with per-call headers. |
| `post_json_to_api` | POST JSON and parse a JSON response. |
| `post_json_to_api_stream` | POST JSON and parse SSE events. |
| `post_json_to_api_bytes` | POST JSON and return bytes plus response headers. |
| `post_form_to_api` | POST multipart form data and parse JSON. |
| `parse_provider_options` | Read provider-scoped options from the shared options map. |

## `ai-provider-openai`

OpenAI provider implementation. Construct `OpenAIProvider`, create a model with a model factory method, then call a function from `ai-core`.
