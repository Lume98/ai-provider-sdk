//! Models API 资源测试。
//!
//! 验证 Models API 的请求构造、URL 编码和参数校验：
//! - 模型列表查询
//! - 包含特殊字符的模型 ID 路径编码
//! - 空 model ID 参数校验

use crate::common::{path_capture_server, test_client};
use httpmock::prelude::*;
use ai_provider_sdk::Error;
use serde_json::json;

#[tokio::test]
/// 验证模型列表查询发送了正确的 GET 请求。
async fn models_list_sends_expected_request() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/models")
            .query_param("api-version", "test")
            .header("authorization", "Bearer sk-test");
        then.status(200).json_body(json!({
            "object": "list",
            "data": [
                {
                    "id": "gpt-4.1-mini",
                    "object": "model",
                    "created": 1710000000,
                    "owned_by": "openai"
                }
            ]
        }));
    });

    let client = test_client(&server);
    let models = client.models().list().await.unwrap();

    mock.assert();
    assert_eq!(models.object.as_deref(), Some("list"));
    assert_eq!(models.data[0].id, "gpt-4.1-mini");
}

#[tokio::test]
/// 验证包含 `/` 和空格的模型 ID 被正确 URL 编码。
///
/// `"fine/tuned model"` → `/models/fine%2Ftuned%20model`
async fn models_retrieve_url_encodes_model_id() {
    let (base_url, path_seen) = path_capture_server(
        "HTTP/1.1 200 OK",
        "{\"id\":\"fine/tuned model\",\"object\":\"model\"}",
    )
    .await;

    let client = ai_provider_sdk::OpenAI::with_options(ai_provider_sdk::ClientOptions {
        api_key: Some("sk-test".to_string()),
        organization: Some("org_123".to_string()),
        project: Some("proj_123".to_string()),
        base_url: Some(base_url),
        default_headers: Some(std::collections::HashMap::from([("x-custom".to_string(), "custom".to_string())])),
        default_query: Some(std::collections::HashMap::from([("api-version".to_string(), "test".to_string())])),
        max_retries: 0,
        timeout: Some(std::time::Duration::from_secs(5)),
        ..ai_provider_sdk::ClientOptions::default()
    })
    .unwrap();

    let model = client.models().retrieve("fine/tuned model").await.unwrap();

    // 验证路径中的 `/` 和空格被正确编码
    assert_eq!(
        path_seen.lock().unwrap().as_deref(),
        Some("/models/fine%2Ftuned%20model?api-version=test")
    );
    // 验证响应体中的原始 ID 未被编码影响
    assert_eq!(model.id, "fine/tuned model");
}

#[tokio::test]
/// 验证空 model ID 被客户端拦截，返回配置错误。
async fn models_retrieve_rejects_empty_id() {
    let server = MockServer::start();
    let client = test_client(&server);

    let err = client.models().retrieve("").await.unwrap_err();
    match err {
        Error::Config(message) => assert!(message.contains("must not be empty")),
        other => panic!("unexpected error: {other:?}"),
    }
}
