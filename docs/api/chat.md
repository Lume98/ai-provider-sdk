# Chat Completions

## 创建

```rust
use openai_rust::{ChatCompletionCreateParams, ChatMessage, OpenAI};

# async fn demo() -> Result<(), openai_rust::Error> {
let client = OpenAI::from_env()?;
let completion = client
    .chat()
    .completions()
    .create(ChatCompletionCreateParams::new(
        "gpt-4.1-mini",
        vec![ChatMessage::user("hello")],
    ))
    .await?;

println!("{}", completion.id);
# Ok(())
# }
```

## 已实现方法

- `create`
- `create_with_options`
- `create_stream`
- `create_stream_with_options`
