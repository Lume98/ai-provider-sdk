# 快速开始

## 安装文档依赖

```bash
pnpm install
```

## 启动文档

```bash
pnpm docs:dev
```

## SDK 最小示例

```rust
use vendor_sdk::{ChatCompletionCreateParams, ChatMessage, OpenAIClient};

# async fn demo() -> Result<(), vendor_sdk::Error> {
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
