use crate::common::test_client;
use httpmock::prelude::*;
use openai_rust::{ModerationCreateParams, ModerationInputItem};
use serde_json::json;

#[tokio::test]
async fn moderations_create_sends_expected_request() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/moderations").json_body(json!({
            "input": "I want to hurt them",
            "model": "omni-moderation-latest"
        }));
        then.status(200).json_body(json!({
            "id": "modr_123",
            "model": "omni-moderation-latest",
            "results": [
                {
                    "flagged": true,
                    "categories": {
                        "harassment": true,
                        "harassment/threatening": true,
                        "violence": false,
                        "self-harm": false,
                        "illicit": false,
                        "illicit/violent": false
                    },
                    "category_scores": {
                        "harassment": 0.91,
                        "harassment/threatening": 0.82,
                        "violence": 0.12,
                        "self-harm": 0.01,
                        "illicit": 0.02,
                        "illicit/violent": 0.0
                    }
                }
            ]
        }));
    });

    let client = test_client(&server);
    let response = client
        .moderations()
        .create(ModerationCreateParams::new("I want to hurt them").model("omni-moderation-latest"))
        .await
        .unwrap();

    mock.assert();
    assert_eq!(response.id, "modr_123");
    assert!(response.results[0].flagged);
    assert_eq!(response.results[0].categories.harassment_threatening, Some(true));
    assert_eq!(response.results[0].category_scores.harassment, Some(0.91));
}

#[tokio::test]
async fn moderations_create_supports_multimodal_input_items() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/moderations").json_body(json!({
            "input": [
                {"type": "text", "text": "check this image"},
                {"type": "image_url", "image_url": {"url": "https://example.com/image.png"}}
            ]
        }));
        then.status(200).json_body(json!({
            "id": "modr_456",
            "model": "omni-moderation-latest",
            "results": [
                {
                    "flagged": false,
                    "categories": {
                        "sexual": false,
                        "violence/graphic": false
                    },
                    "category_scores": {
                        "sexual": 0.0,
                        "violence/graphic": 0.0
                    },
                    "category_applied_input_types": {
                        "sexual": ["text", "image"],
                        "violence/graphic": ["image"]
                    }
                }
            ]
        }));
    });

    let client = test_client(&server);
    let response = client
        .moderations()
        .create(ModerationCreateParams::new(vec![
            ModerationInputItem::text("check this image"),
            ModerationInputItem::image_url("https://example.com/image.png"),
        ]))
        .await
        .unwrap();

    mock.assert();
    assert_eq!(response.id, "modr_456");
    assert_eq!(response.results[0].categories.sexual, Some(false));
    assert_eq!(
        response.results[0]
            .category_applied_input_types
            .as_ref()
            .unwrap()
            .violence_graphic
            .as_ref()
            .unwrap()
            .len(),
        1
    );
}
