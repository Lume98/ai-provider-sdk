# Responses

## 创建响应

```rust
use vendor_sdk::{OpenAIClient, ResponseCreateParams};

# async fn demo() -> Result<(), vendor_sdk::Error> {
let client = OpenAIClient::from_env()?;
let response = client
    .responses
    .create(&ResponseCreateParams::new("gpt-4.1-mini", "hello"))
    .await?;
# Ok(())
# }
```

## 已实现方法

- `create(&ResponseCreateParams)`
- `stream(&ResponseCreateParams)`
- `retrieve(&str)`
- `delete(&str)`

`ResponseCreateParams` 支持 `tools`、`reasoning`、`metadata`、`max_output_tokens` 等字段，并通过 `extra` 扩展未显式建模参数。
