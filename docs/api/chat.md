# Chat Completions

## 创建对话

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
# Ok(())
# }
```

## 已实现方法

- `create(&ChatCompletionCreateParams)`
- `create_stream(&ChatCompletionCreateParams)`

`ChatMessage` 已支持 `developer/system/user/assistant/tool` 角色，并支持工具调用结构（`ChatTool`、`ChatToolCall`）。
