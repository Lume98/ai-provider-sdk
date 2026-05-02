//! Responses API 资源测试。
//!
//! 验证 Responses API 的请求构造和 RequestOptions 覆盖行为：
//! - 基本创建请求（model、input、stream 字段）
//! - RequestOptions 覆盖查询参数、请求头和请求体

use crate::common::test_client;
use httpmock::prelude::*;
use ai_provider_sdk::{RequestOptions, ResponseCreateParams};
use serde_json::json;

#[tokio::test]
/// 验证 Responses API 创建请求发送了正确的请求体和头信息。
async fn responses_create_sends_expected_request() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/responses")
            .query_param("api-version", "test")
            .header("authorization", "Bearer sk-test")
            .header("openai-organization", "org_123")
            .header("openai-project", "proj_123")
            .header("x-custom", "custom")
            .json_body(json!({
                "model": "gpt-4.1-mini",
                "input": "hello",
                "stream": false
            }));
        then.status(200).json_body(json!({
            "id": "resp_123",
            "object": "response"
        }));
    });

    let client = test_client(&server);
    let response = client
        .responses()
        .create(ResponseCreateParams::new("gpt-4.1-mini").input("hello"))
        .await
        .unwrap();

    mock.assert();
    assert_eq!(response.id, "resp_123");
    assert_eq!(response.extra["object"], "response");
}

#[tokio::test]
/// 验证 RequestOptions 可以覆盖查询参数、请求头和请求体字段。
///
/// - `query("trace", "1")` 追加查询参数
/// - `header("x-extra", "yes")` 追加请求头
/// - `extra_body(json!({"model": "override-model"}))` 覆盖请求体中的 model 字段
async fn request_options_override_query_header_and_body() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/responses")
            .query_param("api-version", "test")
            .query_param("trace", "1")
            .header("x-extra", "yes")
            .json_body(json!({
                "model": "override-model",
                "input": "hello",
                "stream": false
            }));
        then.status(200).json_body(json!({"id": "resp_123"}));
    });

    let client = test_client(&server);
    client
        .responses()
        .create_with_options(
            ResponseCreateParams::new("gpt-4.1-mini").input("hello"),
            RequestOptions::new()
                .header("x-extra", "yes")
                .query("trace", "1")
                .extra_body(json!({"model": "override-model"})),
        )
        .await
        .unwrap();

    mock.assert();
}
