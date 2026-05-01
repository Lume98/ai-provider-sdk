# 快速开始

本页只保留“从 0 到 1 跑通”的最小步骤。完整安装与使用路线见 [/guide/overview](/guide/overview)。

## 1. 安装

```bash
cargo add ai-provider-sdk
cargo add tokio --features macros,rt-multi-thread
```

## 2. 配置

```bash
export OPENAI_API_KEY="sk-..."
```

## 3. 发送第一个请求（Responses）

```rust
use ai_provider_sdk::{OpenAI, ResponseCreateParams};

#[tokio::main]
async fn main() -> Result<(), ai_provider_sdk::Error> {
    let client = OpenAI::from_env()?;

    let response = client
        .responses()
        .create(ResponseCreateParams::new("gpt-4.1-mini").input("hello"))
        .await?;

    println!("{}", response.id);
    Ok(())
}
```

## 4. 流式请求（Chat Completions）

前面的 Responses 示例展示了非流式调用，下面演示 Chat Completions 的流式用法。SDK 中所有支持流式的资源均采用相同的 SSE 模式。

```rust
use futures_util::StreamExt;
use ai_provider_sdk::{ChatCompletionCreateParams, ChatMessage, OpenAI};

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
    println!("event={:?}, data={}", event.event, event.data);
}
```

## 5. 下一步

- 配置优先级与 `RequestOptions`：[/guide/configuration](/guide/configuration)
- 资源能力矩阵：[/api/resources](/api/resources)
