# Webhooks

通过 `client.webhooks.verify_signature(...)` 做签名校验：

```rust
use reqwest::header::HeaderMap;
use vendor_ai_sdk::OpenAIClient;

fn verify(headers: &HeaderMap, body: &[u8]) -> Result<(), vendor_ai_sdk::Error> {
    let client = OpenAIClient::new("sk-unused");
    client.webhooks.verify_signature("whsec_...", body, headers)
}
```

实现要点：

- 支持读取 `webhook-signature` / `OpenAI-Signature`
- 支持读取 `webhook-timestamp` / `OpenAI-Timestamp`
- 使用 `HMAC-SHA256(secret, "<timestamp>.<payload>")`
- 常量时间比较避免时序攻击
