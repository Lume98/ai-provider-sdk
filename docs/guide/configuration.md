# 配置

## 初始化顺序

`OpenAI::with_options` 的读取优先级：

1. 显式 `ClientOptions`
2. 环境变量
3. 内置默认值

## 环境变量

- `OPENAI_API_KEY`（必需，除非显式传入 `api_key`）
- `OPENAI_BASE_URL`（默认 `https://api.openai.com/v1`）
- `OPENAI_ORG_ID`
- `OPENAI_PROJECT_ID`

## 显式配置

```rust
use std::collections::HashMap;
use std::time::Duration;
use openai_rust::{ClientOptions, OpenAI};

# fn demo() -> Result<(), openai_rust::Error> {
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

## 单次请求覆盖

通过 `RequestOptions` 追加本次请求参数：

- `header(key, value)`
- `query(key, value)`
- `extra_body(json)`
- `timeout(duration)`
