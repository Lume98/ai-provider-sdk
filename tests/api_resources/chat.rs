use crate::common::test_client;
use futures_util::StreamExt;
use httpmock::prelude::*;
use ai_provider_sdk::{ChatCompletionCreateParams, ChatMessage};
use serde_json::json;

#[tokio::test]
async fn chat_completions_create_sends_expected_request() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/chat/completions")
            .json_body(json!({
                "model": "gpt-4.1-mini",
                "messages": [{"role": "user", "content": "hello"}],
                "stream": false
            }));
        then.status(200).json_body(json!({
            "id": "chatcmpl_123",
            "choices": []
        }));
    });

    let client = test_client(&server);
    let completion = client
        .chat()
        .completions()
        .create(ChatCompletionCreateParams::new(
            "gpt-4.1-mini",
            vec![ChatMessage::user("hello")],
        ))
        .await
        .unwrap();

    mock.assert();
    assert_eq!(completion.id, "chatcmpl_123");
    assert_eq!(completion.extra["choices"], json!([]));
}

#[tokio::test]
async fn streams_sse_events_until_done() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/chat/completions")
            .json_body(json!({
                "model": "gpt-4.1-mini",
                "messages": [{"role": "user", "content": "hello"}],
                "stream": true
            }));
        then.status(200)
            .header("content-type", "text/event-stream")
            .body("data: {\"id\":\"chunk_1\"}\n\ndata: [DONE]\n\n");
    });

    let client = test_client(&server);
    let stream = client
        .chat()
        .completions()
        .create_stream(ChatCompletionCreateParams::new(
            "gpt-4.1-mini",
            vec![ChatMessage::user("hello")],
        ))
        .await
        .unwrap();

    let events: Vec<_> = stream
        .events()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();

    mock.assert();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data, "{\"id\":\"chunk_1\"}");
}
