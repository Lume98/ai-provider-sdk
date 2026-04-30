# Streaming

SDK 的 SSE 接口返回 `TypedSseStream<T>`。

## Responses 流式

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

## Chat Completions 流式

使用 `client.chat.completions.create_stream(...)`，事件类型为 `ChatCompletionChunk`。
