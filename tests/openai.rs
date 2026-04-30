use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::{Value, json};
use vendor_ai_sdk::{
    ChatCompletionCreateParams, ChatMessage, Error, FileListParams, ListParams, OpenAIClient,
    OpenAIConfig, ResponseCreateParams,
};

#[derive(Debug)]
struct CapturedRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Value,
}

struct MockServer {
    base_url: String,
    handle: thread::JoinHandle<CapturedRequest>,
}

impl MockServer {
    fn once(status: u16, content_type: &'static str, body: &'static str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
        let base_url = format!(
            "http://{}",
            listener.local_addr().expect("mock server addr")
        );
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let request = read_request(&mut stream);
            let response = format!(
                "HTTP/1.1 {status} OK\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("write mock response");
            request
        });

        Self { base_url, handle }
    }

    fn captured(self) -> CapturedRequest {
        self.handle.join().expect("mock server thread")
    }
}

fn read_request(stream: &mut TcpStream) -> CapturedRequest {
    let mut buf = Vec::new();
    let mut tmp = [0_u8; 1024];
    let header_end = loop {
        let read = stream.read(&mut tmp).expect("read request");
        assert_ne!(read, 0, "connection closed before headers");
        buf.extend_from_slice(&tmp[..read]);
        if let Some(pos) = find_header_end(&buf) {
            break pos;
        }
    };

    let header_text = String::from_utf8_lossy(&buf[..header_end]).to_string();
    let mut lines = header_text.split("\r\n");
    let request_line = lines.next().expect("request line");
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().expect("method").to_string();
    let path = request_parts.next().expect("path").to_string();
    let mut headers = HashMap::new();

    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        headers.insert(name.to_ascii_lowercase(), value.trim().to_string());
    }

    let content_length = headers
        .get("content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    let body_start = header_end + 4;
    while buf.len() < body_start + content_length {
        let read = stream.read(&mut tmp).expect("read request body");
        assert_ne!(read, 0, "connection closed before body");
        buf.extend_from_slice(&tmp[..read]);
    }
    let body = if content_length == 0 {
        Value::Null
    } else if headers
        .get("content-type")
        .is_some_and(|value| value.starts_with("application/json"))
    {
        serde_json::from_slice(&buf[body_start..body_start + content_length]).expect("json body")
    } else {
        Value::String(String::from_utf8_lossy(&buf[body_start..body_start + content_length]).into())
    };

    CapturedRequest {
        method,
        path,
        headers,
        body,
    }
}

fn find_header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|window| window == b"\r\n\r\n")
}

fn client(base_url: &str) -> OpenAIClient {
    OpenAIClient::from_config(
        OpenAIConfig::new("sk-test")
            .with_base_url(base_url)
            .with_organization("org-test")
            .with_project("proj-test")
            .with_max_retries(0),
    )
}

#[tokio::test]
async fn chat_completions_create_posts_to_resource_tree_path() {
    let server = MockServer::once(
        200,
        "application/json",
        r#"{
            "id":"chatcmpl_123",
            "object":"chat.completion",
            "created":1710000000,
            "model":"gpt-5.4",
            "choices":[{"index":0,"message":{"role":"assistant","content":"pong"},"finish_reason":"stop"}],
            "usage":{"prompt_tokens":3,"completion_tokens":2,"total_tokens":5}
        }"#,
    );
    let mut req = ChatCompletionCreateParams::new("gpt-5.4", vec![ChatMessage::developer("ping")]);
    req.temperature = Some(1.0);
    req.max_completion_tokens = Some(16);
    req.response_format = Some(json!({"type": "text"}));
    req.tool_choice = Some(json!("auto"));
    req.stream = true;

    let response = client(&server.base_url)
        .chat
        .completions
        .create(&req)
        .await
        .expect("chat");
    let captured = server.captured();

    assert_eq!(captured.method, "POST");
    assert_eq!(captured.path, "/chat/completions");
    assert_eq!(captured.headers["authorization"], "Bearer sk-test");
    assert_eq!(captured.headers["openai-organization"], "org-test");
    assert_eq!(captured.headers["openai-project"], "proj-test");
    assert_eq!(captured.body["model"], "gpt-5.4");
    assert_eq!(captured.body["messages"][0]["role"], "developer");
    assert_eq!(captured.body["temperature"], 1.0);
    assert_eq!(captured.body["max_completion_tokens"], 16);
    assert_eq!(captured.body["tool_choice"], "auto");
    assert!(captured.body.get("stream").is_none());
    assert_eq!(response.choices[0].message.content.as_deref(), Some("pong"));
}

#[tokio::test]
async fn responses_stream_posts_stream_true_and_collects_events() {
    let server = MockServer::once(
        200,
        "text/event-stream",
        concat!(
            "event: response.created\n",
            "data: {\"type\":\"response.created\",\"sequence_number\":0,\"response\":{\"id\":\"resp_123\"}}\n\n",
            "event: response.output_text.delta\n",
            "data: {\"type\":\"response.output_text.delta\",\"sequence_number\":1,\"delta\":\"hi\"}\n\n",
            "data: [DONE]\n\n"
        ),
    );
    let req = ResponseCreateParams::new("gpt-5.1", "hi");

    let events = client(&server.base_url)
        .responses
        .stream(&req)
        .await
        .expect("responses stream")
        .collect_all()
        .await
        .expect("collect events");
    let captured = server.captured();

    assert_eq!(captured.path, "/responses");
    assert_eq!(captured.body["stream"], true);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event_type, "response.created");
    assert_eq!(events[1].data["delta"], "hi");
}

#[tokio::test]
async fn files_list_serializes_query_params() {
    let server = MockServer::once(
        200,
        "application/json",
        r#"{"object":"list","data":[{"id":"file_123","object":"file","purpose":"assistants"}],"has_more":false}"#,
    );
    let params = FileListParams {
        page: ListParams {
            limit: Some(10),
            ..ListParams::default()
        },
        purpose: Some("assistants".into()),
    };

    let page = client(&server.base_url)
        .files
        .list(&params)
        .await
        .expect("files list");
    let captured = server.captured();

    assert_eq!(captured.method, "GET");
    assert!(
        captured.path == "/files?limit=10&purpose=assistants"
            || captured.path == "/files?purpose=assistants&limit=10"
    );
    assert_eq!(page.data[0].id, "file_123");
}

#[tokio::test]
async fn api_error_response_uses_openai_error_code_and_message() {
    let server = MockServer::once(
        429,
        "application/json",
        r#"{"error":{"message":"rate limit exceeded","code":"rate_limit_exceeded","type":"requests"}}"#,
    );
    let req = ChatCompletionCreateParams::new("gpt-5.4", vec![ChatMessage::user("hi")]);

    let err = client(&server.base_url)
        .chat
        .completions
        .create(&req)
        .await
        .expect_err("api error");
    let captured = server.captured();

    assert_eq!(captured.path, "/chat/completions");
    assert!(matches!(
        err,
        Error::Api {
            status: 429,
            code,
            message,
            error_type: Some(error_type),
            ..
        } if code == "rate_limit_exceeded" && message == "rate limit exceeded" && error_type == "requests"
    ));
}

#[test]
fn webhook_signature_verification_accepts_openai_style_headers() {
    let client = OpenAIClient::new("sk-test");
    let mut headers = HeaderMap::new();
    headers.insert("webhook-timestamp", HeaderValue::from_static("1710000000"));
    headers.insert(
        "webhook-signature",
        HeaderValue::from_static(
            "v1=2edd4001be3556ef3ef9980e57465756f4ae952aaffb78ee8f6bac3465579081",
        ),
    );

    client
        .webhooks
        .verify_signature("whsec_test", br#"{"id":"evt_123"}"#, &headers)
        .expect("valid signature");
}
