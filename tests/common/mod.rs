use httpmock::prelude::*;
use openai_rust::{ClientOptions, FileObject, OpenAI};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub fn test_client(server: &MockServer) -> OpenAI {
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

pub fn test_client_with_base_url(base_url: String) -> OpenAI {
    OpenAI::with_options(ClientOptions {
        api_key: Some("sk-test".to_string()),
        base_url: Some(base_url),
        default_query: HashMap::from([("api-version".to_string(), "test".to_string())]),
        max_retries: 0,
        timeout: Duration::from_secs(5),
        ..ClientOptions::default()
    })
    .unwrap()
}

pub async fn retry_server() -> (String, Arc<Mutex<Vec<String>>>) {
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

pub async fn path_capture_server(
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

pub async fn request_capture_server(
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

pub async fn request_path_sequence_server(bodies: Vec<String>) -> (String, Arc<Mutex<Vec<String>>>) {
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

pub fn file_object(id: &str) -> FileObject {
    serde_json::from_value(json!({
        "id": id,
        "object": "file",
        "filename": format!("{id}.jsonl"),
        "purpose": "fine-tune"
    }))
    .unwrap()
}
