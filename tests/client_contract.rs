use bytes::Bytes;
use futures_util::StreamExt;
use httpmock::prelude::*;
use openai_rust::{
    ChatCompletionCreateParams, ChatMessage, ClientOptions, CursorPage, EmbeddingCreateParams,
    EmbeddingVector, Error, FileCreateParams, FileListParams, FileObject, FilePurpose, ListOrder,
    ModerationCreateParams, ModerationInputItem, OpenAI, RequestOptions, ResponseCreateParams,
    UploadFile,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn test_client(server: &MockServer) -> OpenAI {
    OpenAI::with_options(ClientOptions {
        api_key: Some("sk-test".to_string()),
        organization: Some("org_123".to_string()),
        project: Some("proj_123".to_string()),
        base_url: Some(server.base_url()),
        default_headers: HashMap::from([("x-custom".to_string(), "custom".to_string())]),
        default_query: HashMap::from([("api-version".to_string(), "test".to_string())]),
        max_retries: 0,
        timeout: Duration::from_secs(5),
    })
    .unwrap()
}

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

    let client = OpenAI::with_options(ClientOptions {
        api_key: Some("sk-test".to_string()),
        organization: Some("org_123".to_string()),
        project: Some("proj_123".to_string()),
        base_url: Some(base_url),
        default_headers: HashMap::from([("x-custom".to_string(), "custom".to_string())]),
        default_query: HashMap::from([("api-version".to_string(), "test".to_string())]),
        max_retries: 0,
        timeout: Duration::from_secs(5),
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
        .create(EmbeddingCreateParams::new(
            "text-embedding-3-small",
            "hello",
        ))
        .await
        .unwrap();

    mock.assert();
    assert_eq!(
        response.data[0].embedding,
        EmbeddingVector::Float(vec![0.1, 0.2])
    );
    assert_eq!(response.usage.unwrap().total_tokens, 1);
}

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
    assert_eq!(
        response.results[0].categories.harassment_threatening,
        Some(true)
    );
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

#[tokio::test]
async fn files_create_sends_multipart_upload() {
    let (base_url, request_seen) = request_capture_server(
        "HTTP/1.1 200 OK",
        "{\"id\":\"file_123\",\"object\":\"file\",\"filename\":\"train.jsonl\",\"purpose\":\"fine-tune\"}",
    )
    .await;

    let client = OpenAI::with_options(ClientOptions {
        api_key: Some("sk-test".to_string()),
        base_url: Some(base_url),
        default_query: HashMap::from([("api-version".to_string(), "test".to_string())]),
        max_retries: 0,
        timeout: Duration::from_secs(5),
        ..ClientOptions::default()
    })
    .unwrap();

    let file = client
        .files()
        .create(FileCreateParams::new(
            UploadFile::from_bytes(
                "train.jsonl",
                Bytes::from_static(br#"{"prompt":"hi","completion":"there"}"#),
            ),
            FilePurpose::FineTune,
        ))
        .await
        .unwrap();

    let request = request_seen.lock().unwrap().clone().unwrap();
    let lower = request.to_ascii_lowercase();
    assert!(request.starts_with("POST /files?api-version=test HTTP/1.1"));
    assert!(lower.contains("content-type: multipart/form-data; boundary="));
    assert!(request.contains("name=\"purpose\""));
    assert!(request.contains("fine-tune"));
    assert!(request.contains("name=\"file\"; filename=\"train.jsonl\""));
    assert!(request.contains(r#"{"prompt":"hi","completion":"there"}"#));
    assert_eq!(file.id, "file_123");
}

#[tokio::test]
async fn files_list_returns_cursor_page() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/files")
            .query_param("limit", "2")
            .query_param("order", "desc")
            .query_param("purpose", "fine-tune");
        then.status(200).json_body(json!({
            "object": "list",
            "data": [
                {"id": "file_1", "object": "file", "filename": "a.jsonl", "purpose": "fine-tune"},
                {"id": "file_2", "object": "file", "filename": "b.jsonl", "purpose": "fine-tune"}
            ],
            "has_more": true
        }));
    });

    let client = test_client(&server);
    let mut params = FileListParams::new();
    params.limit = Some(2);
    params.order = Some(ListOrder::Desc);
    params.purpose = Some("fine-tune".to_string());

    let page = client.files().list_with_params(params).await.unwrap();

    mock.assert();
    assert!(page.has_next_page());
    assert_eq!(page.next_after(), Some("file_2"));
    assert_eq!(page.items().len(), 2);
}

#[tokio::test]
async fn files_list_next_page_uses_last_item_cursor() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/files")
            .query_param("after", "file_2")
            .query_param("limit", "2");
        then.status(200).json_body(json!({
            "object": "list",
            "data": [
                {"id": "file_3", "object": "file", "filename": "c.jsonl", "purpose": "fine-tune"}
            ],
            "has_more": false
        }));
    });

    let client = test_client(&server);
    let mut params = FileListParams::new();
    params.limit = Some(2);
    let current_page = CursorPage {
        object: Some("list".to_string()),
        data: vec![file_object("file_1"), file_object("file_2")],
        has_more: Some(true),
        extra: HashMap::new(),
    };

    let next_page = client
        .files()
        .list_next_page(&current_page, params)
        .await
        .unwrap()
        .unwrap();

    mock.assert();
    assert_eq!(next_page.items()[0].id, "file_3");
    assert!(!next_page.has_next_page());
}

#[tokio::test]
async fn files_list_auto_paging_streams_items_across_pages() {
    let (base_url, paths_seen) = request_path_sequence_server(vec![
        json!({
            "object": "list",
            "data": [
                {"id": "file_1", "object": "file", "filename": "a.jsonl", "purpose": "fine-tune"},
                {"id": "file_2", "object": "file", "filename": "b.jsonl", "purpose": "fine-tune"}
            ],
            "has_more": true
        })
        .to_string(),
        json!({
            "object": "list",
            "data": [
                {"id": "file_3", "object": "file", "filename": "c.jsonl", "purpose": "fine-tune"}
            ],
            "has_more": false
        })
        .to_string(),
    ])
    .await;

    let client = OpenAI::with_options(ClientOptions {
        api_key: Some("sk-test".to_string()),
        base_url: Some(base_url),
        default_query: HashMap::from([("api-version".to_string(), "test".to_string())]),
        max_retries: 0,
        timeout: Duration::from_secs(5),
        ..ClientOptions::default()
    })
    .unwrap();
    let mut params = FileListParams::new();
    params.limit = Some(2);

    let items: Vec<FileObject> = client
        .files()
        .list_auto_paging(params)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap();
    let paths = paths_seen.lock().unwrap();

    assert_eq!(
        items
            .iter()
            .map(|file| file.id.as_str())
            .collect::<Vec<_>>(),
        vec!["file_1", "file_2", "file_3"]
    );
    assert_eq!(paths.len(), 2);
    assert!(paths[0].starts_with("/files?"));
    assert!(!paths[0].contains("after="));
    assert!(paths[0].contains("limit=2"));
    assert!(paths[1].starts_with("/files?"));
    assert!(paths[1].contains("after=file_2"));
    assert!(paths[1].contains("limit=2"));
}

#[tokio::test]
async fn files_content_returns_binary_response() {
    let (base_url, request_seen) =
        request_capture_server("HTTP/1.1 200 OK", "raw file bytes").await;

    let client = OpenAI::with_options(ClientOptions {
        api_key: Some("sk-test".to_string()),
        base_url: Some(base_url),
        default_query: HashMap::from([("api-version".to_string(), "test".to_string())]),
        max_retries: 0,
        timeout: Duration::from_secs(5),
        ..ClientOptions::default()
    })
    .unwrap();

    let bytes = client.files().content("file/123").await.unwrap();

    let request = request_seen.lock().unwrap().clone().unwrap();
    assert!(request.starts_with("GET /files/file%2F123/content?api-version=test HTTP/1.1"));
    assert!(request
        .to_ascii_lowercase()
        .contains("accept: application/binary"));
    assert_eq!(bytes, Bytes::from_static(b"raw file bytes"));
}

#[tokio::test]
async fn files_retrieve_and_delete_use_file_id() {
    let server = MockServer::start();
    let retrieve = server.mock(|when, then| {
        when.method(GET).path("/files/file_123");
        then.status(200).json_body(json!({
            "id": "file_123",
            "object": "file",
            "filename": "train.jsonl",
            "purpose": "fine-tune"
        }));
    });
    let delete = server.mock(|when, then| {
        when.method(DELETE).path("/files/file_123");
        then.status(200).json_body(json!({
            "id": "file_123",
            "object": "file",
            "deleted": true
        }));
    });

    let client = test_client(&server);
    let file = client.files().retrieve("file_123").await.unwrap();
    let deleted = client.files().delete("file_123").await.unwrap();

    retrieve.assert();
    delete.assert();
    assert_eq!(file.id, "file_123");
    assert!(deleted.deleted);
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
            assert_eq!(
                body.unwrap()["error"]["type"],
                json!("invalid_request_error")
            );
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn retries_retryable_statuses_with_same_idempotency_key() {
    let (base_url, idempotency_keys) = retry_server().await;

    let client = OpenAI::with_options(ClientOptions {
        api_key: Some("sk-test".to_string()),
        base_url: Some(base_url),
        max_retries: 1,
        timeout: Duration::from_secs(5),
        ..ClientOptions::default()
    })
    .unwrap();

    let response = client
        .responses()
        .create(ResponseCreateParams::new("gpt-4.1-mini"))
        .await
        .unwrap();

    assert_eq!(response.id, "resp_retry");
    let keys = idempotency_keys.lock().unwrap();
    assert_eq!(keys.len(), 2);
    assert_eq!(keys[0], keys[1]);
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

async fn retry_server() -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let keys = Arc::new(Mutex::new(Vec::new()));
    let keys_for_task = keys.clone();

    tokio::spawn(async move {
        for attempt in 0..2 {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buffer = vec![0; 8192];
            let read = socket.read(&mut buffer).await.unwrap();
            let request = String::from_utf8_lossy(&buffer[..read]);
            let key = request
                .lines()
                .find_map(|line| {
                    line.strip_prefix("idempotency-key: ")
                        .or_else(|| line.strip_prefix("Idempotency-Key: "))
                })
                .unwrap()
                .to_string();
            keys_for_task.lock().unwrap().push(key);

            let (status, headers, body) = if attempt == 0 {
                (
                    "HTTP/1.1 429 Too Many Requests",
                    "retry-after-ms: 1\r\ncontent-type: application/json",
                    "{\"error\":{\"message\":\"rate limit\"}}",
                )
            } else {
                (
                    "HTTP/1.1 200 OK",
                    "content-type: application/json",
                    "{\"id\":\"resp_retry\"}",
                )
            };

            let response = format!(
                "{status}\r\n{headers}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            socket.write_all(response.as_bytes()).await.unwrap();
        }
    });

    (format!("http://{addr}"), keys)
}

async fn path_capture_server(
    status_line: &str,
    body: &str,
) -> (String, Arc<Mutex<Option<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let path_seen = Arc::new(Mutex::new(None));
    let path_for_task = path_seen.clone();
    let status_line = status_line.to_string();
    let body = body.to_string();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buffer = vec![0; 8192];
        let read = socket.read(&mut buffer).await.unwrap();
        let request = String::from_utf8_lossy(&buffer[..read]);
        let path = request
            .lines()
            .next()
            .unwrap()
            .split_whitespace()
            .nth(1)
            .unwrap()
            .to_string();
        *path_for_task.lock().unwrap() = Some(path);

        let response = format!(
            "{status_line}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
            body.len()
        );
        socket.write_all(response.as_bytes()).await.unwrap();
    });

    (format!("http://{addr}"), path_seen)
}

async fn request_capture_server(
    status_line: &str,
    body: &str,
) -> (String, Arc<Mutex<Option<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let request_seen = Arc::new(Mutex::new(None));
    let request_for_task = request_seen.clone();
    let status_line = status_line.to_string();
    let body = body.to_string();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let request = read_http_request(&mut socket).await;
        *request_for_task.lock().unwrap() = Some(request);

        let response = format!(
            "{status_line}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
            body.len()
        );
        socket.write_all(response.as_bytes()).await.unwrap();
    });

    (format!("http://{addr}"), request_seen)
}

async fn request_path_sequence_server(bodies: Vec<String>) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let paths_seen = Arc::new(Mutex::new(Vec::new()));
    let paths_for_task = paths_seen.clone();

    tokio::spawn(async move {
        for body in bodies {
            let (mut socket, _) = listener.accept().await.unwrap();
            let request = read_http_request(&mut socket).await;
            let path = request
                .lines()
                .next()
                .unwrap()
                .split_whitespace()
                .nth(1)
                .unwrap()
                .to_string();
            paths_for_task.lock().unwrap().push(path);

            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            socket.write_all(response.as_bytes()).await.unwrap();
        }
    });

    (format!("http://{addr}"), paths_seen)
}

async fn read_http_request(socket: &mut tokio::net::TcpStream) -> String {
    let mut data = Vec::new();
    let mut buffer = [0; 4096];

    loop {
        let read = socket.read(&mut buffer).await.unwrap();
        if read == 0 {
            break;
        }
        data.extend_from_slice(&buffer[..read]);

        if let Some(header_end) = find_header_end(&data) {
            let headers = String::from_utf8_lossy(&data[..header_end]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                })
                .unwrap_or(0);
            let body_start = header_end + 4;
            if data.len() >= body_start + content_length {
                break;
            }
        }
    }

    String::from_utf8_lossy(&data).to_string()
}

fn find_header_end(data: &[u8]) -> Option<usize> {
    data.windows(4).position(|window| window == b"\r\n\r\n")
}

fn file_object(id: &str) -> FileObject {
    serde_json::from_value(json!({
        "id": id,
        "object": "file",
        "filename": format!("{id}.jsonl"),
        "purpose": "fine-tune"
    }))
    .unwrap()
}
