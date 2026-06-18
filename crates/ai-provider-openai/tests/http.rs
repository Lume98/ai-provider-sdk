use ai_core::{embed, generate_text, stream_text, transcribe};
use ai_provider::{
    AiError, LanguageMessage, LanguageModelCallOptions, LanguageModelStreamPart, Provider,
    TranscriptionModelCallOptions,
};
use ai_provider_openai::{OpenAIProvider, OpenAIProviderSettings};
use futures::StreamExt;
use serde_json::json;
use wiremock::matchers::{body_json, body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn provider(server: &MockServer) -> OpenAIProvider {
    OpenAIProvider::new(OpenAIProviderSettings {
        api_key: Some("test-key".to_string()),
        base_url: Some(server.uri()),
        ..Default::default()
    })
    .unwrap()
}

#[tokio::test]
async fn validates_url_headers_and_request_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .and(header("authorization", "Bearer test-key"))
        .and(body_json(json!({
            "model": "gpt-test",
            "input": [{ "role": "user", "content": "hello" }],
            "stream": false,
            "max_output_tokens": 10
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "resp_1",
            "model": "gpt-test",
            "output_text": "hi",
            "usage": {
                "input_tokens": 1,
                "output_tokens": 1,
                "total_tokens": 2
            }
        })))
        .mount(&server)
        .await;

    let model = provider(&server).responses("gpt-test");
    let result = generate_text(
        &model,
        LanguageModelCallOptions {
            prompt: vec![LanguageMessage::user("hello")],
            max_output_tokens: Some(10),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert_eq!(
        result.content,
        vec![ai_provider::ContentPart::Text {
            text: "hi".to_string()
        }]
    );
    assert_eq!(result.usage.total_tokens, Some(2));
}

#[tokio::test]
async fn parses_error_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/embeddings"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": { "message": "bad key" }
        })))
        .mount(&server)
        .await;

    let model = provider(&server).embedding("text-embedding");
    let error = embed(&model, vec!["hello".to_string()]).await.unwrap_err();

    assert!(matches!(
        error,
        AiError::ApiCall {
            status: Some(401),
            ..
        }
    ));
}

#[tokio::test]
async fn converts_sse_chunks_to_unified_stream_parts() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(
                    "data: {\"type\":\"response.output_text.delta\",\"item_id\":\"msg_1\",\"delta\":\"hel\"}\n\n\
                     data: {\"type\":\"response.output_text.delta\",\"item_id\":\"msg_1\",\"delta\":\"lo\"}\n\n\
                     data: [DONE]\n\n",
                ),
        )
        .mount(&server)
        .await;

    let model = provider(&server).responses("gpt-test");
    let result = stream_text(
        &model,
        LanguageModelCallOptions {
            prompt: vec![LanguageMessage::user("hello")],
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let parts = result
        .stream
        .collect::<Vec<Result<LanguageModelStreamPart, AiError>>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(
        parts,
        vec![
            LanguageModelStreamPart::StreamStart { warnings: vec![] },
            LanguageModelStreamPart::TextDelta {
                id: "msg_1".to_string(),
                delta: "hel".to_string(),
            },
            LanguageModelStreamPart::TextDelta {
                id: "msg_1".to_string(),
                delta: "lo".to_string(),
            },
        ]
    );
}

#[tokio::test]
async fn validates_transcription_multipart_request() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audio/transcriptions"))
        .and(header("authorization", "Bearer test-key"))
        .and(body_string_contains(r#"name="model""#))
        .and(body_string_contains("gpt-transcribe"))
        .and(body_string_contains(r#"name="language""#))
        .and(body_string_contains("en"))
        .and(body_string_contains(r#"filename="sample.wav""#))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "text": "hello from audio"
        })))
        .mount(&server)
        .await;

    let model = provider(&server).transcription("gpt-transcribe");
    let result = transcribe(
        &model,
        TranscriptionModelCallOptions {
            audio: b"wav-data".to_vec(),
            file_name: "sample.wav".to_string(),
            media_type: Some("audio/wav".to_string()),
            language: Some("en".to_string()),
            prompt: None,
            provider_options: None,
            headers: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(result.text, "hello from audio");
}

#[test]
fn reranking_model_is_unsupported() {
    let provider = OpenAIProvider::new(OpenAIProviderSettings {
        api_key: Some("test-key".to_string()),
        ..Default::default()
    })
    .unwrap();

    let error = match provider.reranking_model("rerank-test") {
        Ok(_) => panic!("expected reranking model to be unsupported"),
        Err(error) => error,
    };

    assert!(matches!(error, AiError::Unsupported(_)));
}
