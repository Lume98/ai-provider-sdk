//! 错误处理行为测试。
//!
//! 验证 SDK 在 API 返回错误时的行为：
//! - `ApiStatus` 错误保留状态码、请求体和 request_id
//! - 错误消息从响应体中正确提取

use crate::common::test_client;
use httpmock::prelude::*;
use ai_provider_sdk::{Error, ResponseCreateParams};
use serde_json::json;

#[tokio::test]
/// 验证 API 返回 400 时，Error::ApiStatus 保留了完整诊断信息。
async fn api_status_error_preserves_status_body_and_request_id() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/responses");
        then.status(400)
            .header("x-request-id", "req_123")
            .json_body(json!({
                "error": {
                    "message": "bad request",
                    "type": "invalid_request_error"
                }
            }));
    });

    let client = test_client(&server);
    let error = client
        .responses()
        .create(ResponseCreateParams::new("gpt-4.1-mini"))
        .await
        .unwrap_err();

    match error {
        Error::ApiStatus {
            message,
            status,
            request_id,
            body,
        } => {
            // 验证错误消息从 error.message 中提取
            assert_eq!(message, "bad request");
            // 验证 HTTP 状态码
            assert_eq!(status.as_u16(), 400);
            // 验证 request_id 从响应头中提取
            assert_eq!(request_id.as_deref(), Some("req_123"));
            // 验证完整错误体保留
            assert_eq!(body.unwrap()["error"]["type"], json!("invalid_request_error"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
