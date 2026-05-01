# 快速开始

## 安装

```bash
cargo add openai-rust
```

## 最小调用（Responses）

```rust
use openai_rust::{OpenAI, ResponseCreateParams};

# async fn demo() -> Result<(), openai_rust::Error> {
let client = OpenAI::from_env()?;
let response = client
    .responses()
    .create(ResponseCreateParams::new("gpt-4.1-mini").input("hello"))
    .await?;

println!("{}", response.id);
# Ok(())
# }
```

## 流式调用（Chat Completions）

```rust
use futures_util::StreamExt;
use openai_rust::{ChatCompletionCreateParams, ChatMessage, OpenAI};

# async fn demo() -> Result<(), openai_rust::Error> {
let client = OpenAI::from_env()?;
let mut events = client
    .chat()
    .completions()
    .create_stream(ChatCompletionCreateParams::new(
        "gpt-4.1-mini",
        vec![ChatMessage::user("hello")],
    ))
    .await?
    .events();

while let Some(event) = events.next().await {
    let event = event?;
    println!("{}", event.data);
}
# Ok(())
# }
```
