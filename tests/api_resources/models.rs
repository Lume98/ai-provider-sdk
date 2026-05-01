use crate::common::{path_capture_server, test_client};
use httpmock::prelude::*;
use openai_rust::Error;
use serde_json::json;

#[tokio::test]
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
async fn models_retrieve_url_encodes_model_id() {
    let (base_url, path_seen) = path_capture_server(
        "HTTP/1.1 200 OK",
        "{\"id\":\"fine/tuned model\",\"object\":\"model\"}",
    )
    .await;

    let client = openai_rust::OpenAI::with_options(openai_rust::ClientOptions {
        api_key: Some("sk-test".to_string()),
        organization: Some("org_123".to_string()),
        project: Some("proj_123".to_string()),
        base_url: Some(base_url),
        default_headers: std::collections::HashMap::from([("x-custom".to_string(), "custom".to_string())]),
        default_query: std::collections::HashMap::from([("api-version".to_string(), "test".to_string())]),
        max_retries: 0,
        timeout: std::time::Duration::from_secs(5),
    })
    .unwrap();

    let model = client.models().retrieve("fine/tuned model").await.unwrap();

    assert_eq!(
        path_seen.lock().unwrap().as_deref(),
        Some("/models/fine%2Ftuned%20model?api-version=test")
    );
    assert_eq!(model.id, "fine/tuned model");
}

#[tokio::test]
async fn models_retrieve_rejects_empty_id() {
    let server = MockServer::start();
    let client = test_client(&server);

    let err = client.models().retrieve("").await.unwrap_err();
    match err {
        Error::Config(message) => assert!(message.contains("must not be empty")),
        other => panic!("unexpected error: {other:?}"),
    }
}
