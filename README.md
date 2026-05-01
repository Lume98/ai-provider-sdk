# ai-provider-sdk

Async-first Rust SDK for OpenAI APIs.

当前仓库只覆盖已实现的 OpenAI 资源：Responses、Chat Completions、Models、Files、Embeddings、Moderations。未在本文列出的资源尚未实现，避免把未来规划误认为可用 API。

## 文档导航

- 安装与使用总览：`docs/guide/overview.md`
- 在线文档入口（本地启动后）：`/guide/overview`
- 资源与方法总览：`docs/api/resources.md`

## 安装

```bash
cargo add ai-provider-sdk
```

## 快速开始

```rust
use ai_provider_sdk::{OpenAI, ResponseCreateParams};

let client = OpenAI::from_env()?;
let response = client
    .responses()
    .create(ResponseCreateParams::new("gpt-4.1-mini").input("hello"))
    .await?;

println!("{}", response.id);
```

## 配置模型

`OpenAI::from_env()` 和 `OpenAI::with_options(...)` 使用同一套配置模型：

- `api_key`：必需；未显式传入时读取 `OPENAI_API_KEY`
- `base_url`：默认 `https://api.openai.com/v1`；可用 `OPENAI_BASE_URL` 覆盖
- `organization`：可选；未显式传入时读取 `OPENAI_ORG_ID`
- `project`：可选；未显式传入时读取 `OPENAI_PROJECT_ID`
- `timeout`：默认 600 秒
- `max_retries`：默认 2
- `default_headers`：客户端级默认 header
- `default_query`：客户端级默认 query

显式配置：

```rust
use std::collections::HashMap;
use std::time::Duration;
use ai_provider_sdk::{ClientOptions, OpenAI};

let client = OpenAI::with_options(ClientOptions {
    api_key: Some("sk-test".to_string()),
    organization: None,
    project: None,
    base_url: Some("https://api.openai.com/v1".to_string()),
    timeout: Duration::from_secs(60),
    max_retries: 2,
    default_headers: HashMap::new(),
    default_query: HashMap::new(),
})?;
```

单次请求可用 `RequestOptions` 追加覆盖项：

```rust
use std::time::Duration;
use ai_provider_sdk::{OpenAI, RequestOptions, ResponseCreateParams};

let client = OpenAI::from_env()?;
let response = client
    .responses()
    .create_with_options(
        ResponseCreateParams::new("gpt-4.1-mini").input("hello"),
        RequestOptions::new()
            .header("x-trace-id", "trace-123")
            .query("api-version", "test")
            .timeout(Duration::from_secs(30)),
    )
    .await?;
```

## 已实现资源

### Responses

- `client.responses().create(params)`
- `client.responses().create_with_options(params, options)`
- `client.responses().create_stream(params)`
- `client.responses().create_stream_with_options(params, options)`

```rust
use ai_provider_sdk::{OpenAI, ResponseCreateParams};

let client = OpenAI::from_env()?;
let response = client
    .responses()
    .create(ResponseCreateParams::new("gpt-4.1-mini").input("hello"))
    .await?;

println!("{}", response.id);
```

### Chat Completions

- `client.chat().completions().create(params)`
- `client.chat().completions().create_with_options(params, options)`
- `client.chat().completions().create_stream(params)`
- `client.chat().completions().create_stream_with_options(params, options)`

```rust
use ai_provider_sdk::{ChatCompletionCreateParams, ChatMessage, OpenAI};

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
```

### Models

- `client.models().list()`
- `client.models().list_with_options(options)`
- `client.models().retrieve(model)`
- `client.models().retrieve_with_options(model, options)`

### Files

- `client.files().create(params)`
- `client.files().create_with_options(params, options)`
- `client.files().retrieve(file_id)`
- `client.files().retrieve_with_options(file_id, options)`
- `client.files().list()`
- `client.files().list_with_params(params)`
- `client.files().list_with_options(params, options)`
- `client.files().list_next_page(current_page, params)`
- `client.files().list_next_page_with_options(current_page, params, options)`
- `client.files().list_auto_paging(params)`
- `client.files().list_auto_paging_with_options(params, options)`
- `client.files().delete(file_id)`
- `client.files().delete_with_options(file_id, options)`
- `client.files().content(file_id)`
- `client.files().content_with_options(file_id, options)`

```rust
use bytes::Bytes;
use ai_provider_sdk::{FileCreateParams, FilePurpose, OpenAI, UploadFile};

let client = OpenAI::from_env()?;
let file = client
    .files()
    .create(FileCreateParams::new(
        UploadFile::from_bytes("train.jsonl", Bytes::from_static(b"{\"messages\":[]}\n")),
        FilePurpose::FineTune,
    ))
    .await?;

println!("{}", file.id);
```

### Embeddings

- `client.embeddings().create(params)`
- `client.embeddings().create_with_options(params, options)`

`EmbeddingCreateParams.encoding_format` 未显式设置时，SDK 会默认发送 `float`。

```rust
use ai_provider_sdk::{EmbeddingCreateParams, OpenAI};

let client = OpenAI::from_env()?;
let response = client
    .embeddings()
    .create(EmbeddingCreateParams::new("text-embedding-3-small", "hello"))
    .await?;

println!("{}", response.data.len());
```

### Moderations

- `client.moderations().create(params)`
- `client.moderations().create_with_options(params, options)`

```rust
use ai_provider_sdk::{ModerationCreateParams, OpenAI};

let client = OpenAI::from_env()?;
let response = client
    .moderations()
    .create(ModerationCreateParams::new("hello").model("omni-moderation-latest"))
    .await?;

println!("{}", response.id);
```

## Streaming

`responses` 和 `chat completions` 的 `create_stream(...)` 返回 `SseStream`。调用 `.events()` 后得到 `Stream<Item = Result<ServerSentEvent>>`。

```rust
use futures_util::StreamExt;
use ai_provider_sdk::{OpenAI, ResponseCreateParams};

let client = OpenAI::from_env()?;
let mut events = client
    .responses()
    .create_stream(ResponseCreateParams::new("gpt-4.1-mini").input("hello"))
    .await?
    .events();

while let Some(event) = events.next().await {
    println!("{}", event?.data);
}
```

流式处理边界：

- 收到 `data: [DONE]` 时结束流。
- 事件 `data` 中包含 `{"error": ...}` 时返回 `Error::Stream`。
- 支持标准 SSE 字段：`event`、`data`、`id`、`retry`。

## 分页

当前自动分页只在 Files 资源上实现。

```rust
use futures_util::StreamExt;
use ai_provider_sdk::{FileListParams, OpenAI};

let client = OpenAI::from_env()?;
let mut stream = Box::pin(client.files().list_auto_paging(FileListParams::default()));

while let Some(file) = stream.next().await {
    println!("{}", file?.id);
}
```

## 错误处理

统一错误类型为 `ai_provider_sdk::Error`：

- `ApiStatus { message, status, request_id, body }`：HTTP 非 2xx 响应
- `Timeout`：请求超时
- `Connection(String)`：网络或连接层异常
- `Config(String)`：配置错误，例如缺失 API key
- `Url(...)`：`base_url` 非法
- `HeaderValue(...)`：header 值非法
- `Json(...)`：JSON 编解码失败
- `Io(...)`：文件 I/O 失败
- `Stream(String)`：SSE 解码或流式事件错误

## 设计边界

- 类型结构保留未知字段：响应类型普遍通过 `extra` 接收 API 返回的新字段。
- 不存在的资源不做动态兜底：如果需要新资源，应新增资源模块、类型和契约测试，而不是在调用侧拼 URL。
- 文件上传使用 multipart；`RequestOptions.extra_body` 在 multipart 请求中必须是 JSON object。
- 路径参数会做 URL path segment 编码，空 ID 会返回 `Error::Config`。

## 开发与验证

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

文档站点命令：

```bash
pnpm docs:dev
pnpm docs:build
pnpm docs:preview
```
