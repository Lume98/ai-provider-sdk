# 配置

## 初始化优先级

`OpenAI::with_options` 的读取优先级：

1. 显式 `ClientOptions`
2. 环境变量
3. 内置默认值

## 环境变量

- `OPENAI_API_KEY`（必需，除非显式传入 `api_key`）
- `OPENAI_BASE_URL`（默认 `https://api.openai.com/v1`）
- `OPENAI_ORG_ID`
- `OPENAI_PROJECT_ID`

## `ClientOptions` 字段

- `api_key: Option<String>`：API Key。
- `organization: Option<String>`：组织 ID。
- `project: Option<String>`：项目 ID。
- `base_url: Option<String>`：基础 URL。
- `timeout: Duration`：请求超时，默认 600 秒。
- `max_retries: u32`：重试次数，默认 2。
- `default_headers: HashMap<String, String>`：客户端默认 headers。
- `default_query: HashMap<String, String>`：客户端默认 query。

## 显式配置示例

```rust
use std::collections::HashMap;
use std::time::Duration;
use ai_provider_sdk::{ClientOptions, OpenAI};

# fn demo() -> Result<(), ai_provider_sdk::Error> {
let mut default_headers = HashMap::new();
default_headers.insert("x-trace-id".to_string(), "demo-trace".to_string());

let client = OpenAI::with_options(ClientOptions {
    api_key: Some("sk-test".to_string()),
    organization: None,
    project: None,
    base_url: Some("https://api.openai.com/v1".to_string()),
    timeout: Duration::from_secs(60),
    max_retries: 2,
    default_headers,
    default_query: HashMap::new(),
})?;
# let _ = client;
# Ok(())
# }
```

## `RequestOptions`（单次请求覆盖）

- `header(key, value)`：追加 header。
- `query(key, value)`：追加 query。
- `extra_body(json)`：追加/覆盖 body 字段。
- `timeout(duration)`：覆盖本次请求超时。

示例：

```rust
use std::time::Duration;
use ai_provider_sdk::{OpenAI, RequestOptions, ResponseCreateParams};

# async fn demo() -> Result<(), ai_provider_sdk::Error> {
let client = OpenAI::from_env()?;

let _response = client
    .responses()
    .create_with_options(
        ResponseCreateParams::new("gpt-4.1-mini").input("hello"),
        RequestOptions::new()
            .header("x-trace-id", "trace-123")
            .query("api-version", "test")
            .timeout(Duration::from_secs(30)),
    )
    .await?;
# Ok(())
# }
```

## 关联阅读

- 安装与使用主线：[/guide/overview](/guide/overview)
- 错误处理：[/guide/errors](/guide/errors)
