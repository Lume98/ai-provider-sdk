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

## 运行案例

项目 `examples/` 提供了与 `openai-python/examples` 对齐的 Rust 案例：

```bash
cargo run --example responses_create
cargo run --example responses_stream
cargo run --example files_create
```

运行前请先在 `examples/common/mod.rs` 中把 `TEST_API_KEY` 和 `TEST_BASE_URL` 改成你的测试值。
