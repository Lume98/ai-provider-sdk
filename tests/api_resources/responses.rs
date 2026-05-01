use crate::common::test_client;
use httpmock::prelude::*;
use openai_rust::{RequestOptions, ResponseCreateParams};
use serde_json::json;

#[tokio::test]
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
