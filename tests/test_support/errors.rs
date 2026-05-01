use crate::common::test_client;
use httpmock::prelude::*;
use openai_rust::{Error, ResponseCreateParams};
use serde_json::json;

#[tokio::test]
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
            assert_eq!(message, "bad request");
            assert_eq!(status.as_u16(), 400);
            assert_eq!(request_id.as_deref(), Some("req_123"));
            assert_eq!(body.unwrap()["error"]["type"], json!("invalid_request_error"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
