# 配置

## 环境变量

`OpenAIClient::from_env()` 会读取以下变量：

- `OPENAI_API_KEY`
- `OPENAI_BASE_URL`
- `OPENAI_ORG_ID`
- `OPENAI_PROJECT_ID`

## 显式配置

```rust
use std::time::Duration;
use vendor_sdk::{OpenAIClient, OpenAIConfig};

let client = OpenAIClient::from_config(
    OpenAIConfig::new("sk-test")
        .with_base_url("https://api.openai.com/v1")
        .with_timeout(Duration::from_secs(60))
        .with_max_retries(2),
);
```

`OpenAIConfig` 还支持：

- `with_organization`
- `with_project`
- `with_default_header`
- `with_default_query`
