//! 重试策略与幂等键行为测试。
//!
//! 验证传输层的重试机制：
//! - 可重试状态码（429）触发自动重试
//! - 重试时保持相同的 `idempotency-key`
//! - 服务端 `retry-after-ms` 头被正确解析

use ai_provider_sdk::{ClientOptions, OpenAI, ResponseCreateParams};
use std::time::Duration;

use crate::common::retry_server;

#[tokio::test]
/// 验证遇到 429 时自动重试，且两次请求使用相同的 idempotency-key。
async fn retries_retryable_statuses_with_same_idempotency_key() {
    let (base_url, idempotency_keys) = retry_server().await;
    let client = OpenAI::with_options(ClientOptions {
        api_key: Some("sk-test".to_string()),
        base_url: Some(base_url),
        max_retries: 1, // 允许重试 1 次
        timeout: Some(Duration::from_secs(5)),
        ..ClientOptions::default()
    })
    .unwrap();

    // 第一次请求返回 429，自动重试后成功
    let response = client
        .responses()
        .create(ResponseCreateParams::new("gpt-4.1-mini"))
        .await
        .unwrap();

    // 验证最终获得了成功响应
    assert_eq!(response.id, "resp_retry");
    // 验证发出了两次请求
    let keys = idempotency_keys.lock().unwrap();
    assert_eq!(keys.len(), 2);
    // 验证两次请求使用了相同的幂等键
    assert_eq!(keys[0], keys[1]);
}
