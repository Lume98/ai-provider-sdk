# Responses

## 创建

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

## 流式

```rust
use futures_util::StreamExt;
use openai_rust::{OpenAI, ResponseCreateParams};

# async fn demo() -> Result<(), openai_rust::Error> {
let client = OpenAI::from_env()?;
let mut events = client
    .responses()
    .create_stream(ResponseCreateParams::new("gpt-4.1-mini").input("hello"))
    .await?
    .events();

while let Some(event) = events.next().await {
    println!("{}", event?.data);
}
# Ok(())
# }
```

## 已实现方法

- `create`
- `create_with_options`
- `create_stream`
- `create_stream_with_options`
