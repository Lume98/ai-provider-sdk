use ai_provider_sdk::{ClientOptions, OpenAI, ResponseCreateParams};
use std::time::Duration;

use crate::common::retry_server;

#[tokio::test]
async fn retries_retryable_statuses_with_same_idempotency_key() {
    let (base_url, idempotency_keys) = retry_server().await;
    let client = OpenAI::with_options(ClientOptions {
        api_key: Some("sk-test".to_string()),
        base_url: Some(base_url),
        max_retries: 1,
        timeout: Duration::from_secs(5),
        ..ClientOptions::default()
    })
    .unwrap();

    let response = client
        .responses()
        .create(ResponseCreateParams::new("gpt-4.1-mini"))
        .await
        .unwrap();

    assert_eq!(response.id, "resp_retry");
    let keys = idempotency_keys.lock().unwrap();
    assert_eq!(keys.len(), 2);
    assert_eq!(keys[0], keys[1]);
}
