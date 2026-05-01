# 安装与使用总览

本文档给出从安装到调用的最短路径，并指向每个资源的完整参数与响应结构。

## 1. 安装

```bash
cargo add openai-rust
```

如果你的项目尚未启用 Tokio 运行时，补充：

```bash
cargo add tokio --features macros,rt-multi-thread
```

## 2. 配置 API Key

```bash
export OPENAI_API_KEY="sk-..."
```

可选环境变量：

- `OPENAI_BASE_URL`（默认 `https://api.openai.com/v1`）
- `OPENAI_ORG_ID`
- `OPENAI_PROJECT_ID`

## 3. 最小可运行示例（Responses）

```rust
use openai_rust::{OpenAI, ResponseCreateParams};

#[tokio::main]
async fn main() -> Result<(), openai_rust::Error> {
    let client = OpenAI::from_env()?;

    let response = client
        .responses()
        .create(ResponseCreateParams::new("gpt-4.1-mini").input("hello"))
        .await?;

    println!("response id: {}", response.id);
    Ok(())
}
```

## 4. 如何使用（主线）

### 4.1 非流式请求（create）

- 构造 `*CreateParams`
- 调用 `create(params)`
- 从响应对象读取字段（最少保证存在 `id`）

### 4.2 流式请求（create_stream）

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
    let event = event?;
    println!("event={:?}, data={}", event.event, event.data);
}
# Ok(())
# }
```

流式结束与错误语义见 [Streaming](/api/streaming)。

### 4.3 单次请求覆盖（RequestOptions）

通过 `RequestOptions` 可为单次请求追加 header、query 或覆盖超时，完整示例见 [/guide/configuration](/guide/configuration)。

## 5. 下一步阅读

- 快速调用：[/guide/getting-started](/guide/getting-started)
- 配置细节：[/guide/configuration](/guide/configuration)
- 资源与方法矩阵：[/api/resources](/api/resources)
- 资源细节：
  - [/api/responses](/api/responses)
  - [/api/chat](/api/chat)
  - [/api/files](/api/files)
  - [/api/models](/api/models)
  - [/api/embeddings](/api/embeddings)
  - [/api/moderations](/api/moderations)

## 6. 边界声明

当前 SDK 只覆盖文档中列出的已实现资源。未列出的 OpenAI 资源在本仓库中未实现。
