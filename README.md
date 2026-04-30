# vendor-ai-sdk

Handwritten Rust SDK for the OpenAI API.

The public API is organized as an OpenAI-style resource tree:

```rust
use vendor_ai_sdk::{ChatCompletionCreateParams, ChatMessage, OpenAIClient};

# async fn demo() -> Result<(), vendor_ai_sdk::Error> {
let client = OpenAIClient::from_env()?;

let response = client
    .chat
    .completions
    .create(&ChatCompletionCreateParams::new(
        "gpt-4.1-mini",
        vec![ChatMessage::user("hello")],
    ))
    .await?;

println!("{:?}", response.choices[0].message.content);
# Ok(())
# }
```

## Configuration

`OpenAIClient::from_env()` reads:

- `OPENAI_API_KEY`
- `OPENAI_BASE_URL`
- `OPENAI_ORG_ID`
- `OPENAI_PROJECT_ID`

Use `OpenAIConfig` for explicit configuration:

```rust
use std::time::Duration;
use vendor_ai_sdk::{OpenAIClient, OpenAIConfig};

let client = OpenAIClient::from_config(
    OpenAIConfig::new("sk-test")
        .with_base_url("https://api.openai.com/v1")
        .with_timeout(Duration::from_secs(60))
        .with_max_retries(2),
);
```

## Resources

Implemented resource entry points include:

- `client.responses.create/stream/retrieve/delete`
- `client.chat.completions.create/create_stream`
- `client.completions.create`
- `client.models.list/retrieve/delete`
- `client.files.create/retrieve/list/delete/content`
- `client.uploads.create/add_part/complete/cancel`
- `client.images.generate/edit`
- `client.audio.speech/transcriptions/translations`
- `client.embeddings.create`
- `client.moderations.create`
- `client.batches`, `client.fine_tuning`, `client.evals`, `client.containers`
- `client.conversations`, `client.vector_stores`, `client.realtime`, `client.webhooks`, `client.skills`, `client.beta`

Less-stable endpoints use typed resource methods with `GenericCreateParams` and `GenericObject`, preserving unknown fields through `extra`.

## Streaming

SSE endpoints return `TypedSseStream<T>`:

```rust
use futures::StreamExt;
use vendor_ai_sdk::{OpenAIClient, ResponseCreateParams};

# async fn demo() -> Result<(), vendor_ai_sdk::Error> {
let client = OpenAIClient::from_env()?;
let mut stream = client
    .responses
    .stream(&ResponseCreateParams::new("gpt-4.1-mini", "hello"))
    .await?;

while let Some(event) = stream.next().await {
    println!("{:?}", event?);
}
# Ok(())
# }
```

## Files

```rust
use bytes::Bytes;
use vendor_ai_sdk::{FileCreateParams, OpenAIClient};

# async fn demo() -> Result<(), vendor_ai_sdk::Error> {
let client = OpenAIClient::from_env()?;
let file = client
    .files
    .create(FileCreateParams::from_bytes(
        "data.jsonl",
        Bytes::from_static(b"{\"messages\":[]}\n"),
        "fine-tune",
    ))
    .await?;
# Ok(())
# }
```

## Webhooks

```rust
use reqwest::header::HeaderMap;
use vendor_ai_sdk::OpenAIClient;

fn verify(headers: &HeaderMap, body: &[u8]) -> Result<(), vendor_ai_sdk::Error> {
    let client = OpenAIClient::new("sk-unused");
    client.webhooks.verify_signature("whsec_...", body, headers)
}
```

## CLI

The crate includes a small `openai` binary for smoke testing:

```bash
cargo run --bin openai -- models:list
cargo run --bin openai -- chat:create gpt-4.1-mini "hello"
cargo run --bin openai -- responses:create gpt-4.1-mini "hello"
```

## Test

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```
