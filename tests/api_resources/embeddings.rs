use crate::common::test_client;
use httpmock::prelude::*;
use ai_provider_sdk::{EmbeddingCreateParams, EmbeddingInput, EmbeddingVector, EncodingFormat};
use serde_json::json;

#[tokio::test]
async fn embeddings_create_sends_default_float_encoding() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/embeddings").json_body(json!({
            "model": "text-embedding-3-small",
            "input": "hello",
            "encoding_format": "float"
        }));
        then.status(200).json_body(json!({
            "object": "list",
            "model": "text-embedding-3-small",
            "data": [
                {
                    "object": "embedding",
                    "index": 0,
                    "embedding": [0.1, 0.2]
                }
            ],
            "usage": {
                "prompt_tokens": 1,
                "total_tokens": 1
            }
        }));
    });

    let client = test_client(&server);
    let response = client
        .embeddings()
        .create(EmbeddingCreateParams::new("text-embedding-3-small", "hello"))
        .await
        .unwrap();

    mock.assert();
    assert_eq!(response.data[0].embedding, EmbeddingVector::Float(vec![0.1, 0.2]));
    assert_eq!(response.usage.unwrap().total_tokens, 1);
}

#[tokio::test]
async fn embeddings_create_preserves_explicit_encoding_format() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/embeddings").json_body(json!({
            "model": "text-embedding-3-small",
            "input": "hello",
            "encoding_format": "base64"
        }));
        then.status(200).json_body(json!({
            "object": "list",
            "model": "text-embedding-3-small",
            "data": [
                {
                    "object": "embedding",
                    "index": 0,
                    "embedding": "AQID"
                }
            ]
        }));
    });

    let client = test_client(&server);
    let mut params = EmbeddingCreateParams::new("text-embedding-3-small", EmbeddingInput::from("hello"));
    params.encoding_format = Some(EncodingFormat::Base64);

    let response = client.embeddings().create(params).await.unwrap();

    mock.assert();
    assert_eq!(response.data[0].embedding, EmbeddingVector::Base64("AQID".to_string()));
}
