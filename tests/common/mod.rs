//! 测试通用工具模块。
//!
//! 提供各测试用例共享的客户端构造函数和 HTTP 模拟服务器辅助工具。
//!
//! ## 核心工具
//!
//! | 凞性                        | 用途                                    |
//! |---------------------------|----------------------------------------|
//! | `test_client`             | 创建连接到 httpmock 的标准测试客户端            |
//! | `test_client_with_base_url` | 创建连接到自定义 URL 的测试客户端            |
//! | `retry_server`            | 模拟重试场景的 TCP 服务器（先返回 429 再返回 200） |
//! | `path_capture_server`     | 捕获请求路径的 TCP 服务器                     |
//! | `request_capture_server`  | 捕获完整 HTTP 请求的 TCP 服务器               |
//! | `request_path_sequence_server` | 按序返回多响应的 TCP 服务器              |
//! | `file_object`             | 快速构造测试用文件对象                        |

use httpmock::prelude::*;
use ai_provider_sdk::{ClientOptions, FileObject, OpenAI};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// 创建连接到 httpmock 的标准测试客户端。
///
/// 预配置了：
/// - API Key: `sk-test`
/// - 组织: `org_123`
/// - 项目: `proj_123`
/// - 默认头: `x-custom: custom`
/// - 默认查询参数: `api-version=test`
/// - 重试: 0 次（测试中不重试）
/// - 超时: 5 秒
pub fn test_client(server: &MockServer) -> OpenAI {
    OpenAI::with_options(ClientOptions {
        api_key: Some("sk-test".to_string()),
        organization: Some("org_123".to_string()),
        project: Some("proj_123".to_string()),
        base_url: Some(server.base_url()),
        default_headers: Some(HashMap::from([("x-custom".to_string(), "custom".to_string())])),
        default_query: Some(HashMap::from([("api-version".to_string(), "test".to_string())])),
        max_retries: 0,
        timeout: Some(Duration::from_secs(5)),
        ..ClientOptions::default()
    })
    .unwrap()
}

/// 创建连接到自定义 base URL 的测试客户端（用于 TCP mock 服务器场景）。
///
/// 仅配置 API Key、查询参数、超时和零重试，不含组织和项目标识。
pub fn test_client_with_base_url(base_url: String) -> OpenAI {
    OpenAI::with_options(ClientOptions {
        api_key: Some("sk-test".to_string()),
        base_url: Some(base_url),
        default_query: Some(HashMap::from([("api-version".to_string(), "test".to_string())])),
        max_retries: 0,
        timeout: Some(Duration::from_secs(5)),
        ..ClientOptions::default()
    })
    .unwrap()
}

/// 启动一个模拟重试场景的 TCP 服务器。
///
/// 行为：
/// - 第一次请求返回 429 + `retry-after-ms: 1`
/// - 第二次请求返回 200 + `{"id": "resp_retry"}`
///
/// 返回服务器地址和捕获到的 idempotency-key 列表。
/// 测试可验证两次请求使用了相同的幂等键。
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
            // 提取 idempotency-key 头（大小写兼容）
            let key = request
                .lines()
                .find_map(|line| {
                    line.strip_prefix("idempotency-key: ")
                        .or_else(|| line.strip_prefix("Idempotency-Key: "))
                })
                .unwrap()
                .to_string();
            keys_for_task.lock().unwrap().push(key);

            // 第一次返回 429，第二次返回 200
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

/// 启动一个只捕获请求路径的 TCP 服务器（仅处理一次请求）。
///
/// 适用于验证 URL 编码行为等场景。
/// 返回服务器地址和捕获到的请求路径。
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
        // 从请求行中提取路径（如 "GET /models/xxx HTTP/1.1" → "/models/xxx"）
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

/// 启动一个捕获完整 HTTP 请求的 TCP 服务器（仅处理一次请求）。
///
/// 适用于验证请求头、multipart body 等场景。
/// 返回服务器地址和捕获到的完整 HTTP 请求文本。
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

/// 启动一个按序返回多个响应的 TCP 服务器（用于自动翻页测试）。
///
/// 每个请求依次返回 `bodies` 中对应的响应体。
/// 返回服务器地址和按序捕获到的请求路径列表。
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

/// 从 TCP socket 读取完整的 HTTP 请求（含请求行、头和 body）。
///
/// 通过解析 `Content-Length` 头确定 body 长度，确保读取完整请求。
async fn read_http_request(socket: &mut tokio::net::TcpStream) -> String {
    let mut data = Vec::new();
    let mut buffer = [0; 4096];

    loop {
        let read = socket.read(&mut buffer).await.unwrap();
        if read == 0 {
            break;
        }
        data.extend_from_slice(&buffer[..read]);

        // 查找请求头结束位置（\r\n\r\n）
        if let Some(header_end) = find_header_end(&data) {
            let headers = String::from_utf8_lossy(&data[..header_end]);
            // 解析 Content-Length 以确定 body 长度
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
            // 等待 body 完整到达
            if data.len() >= body_start + content_length {
                break;
            }
        }
    }

    String::from_utf8_lossy(&data).to_string()
}

/// 在字节数组中查找 HTTP 头结束标记 `\r\n\r\n` 的位置。
fn find_header_end(data: &[u8]) -> Option<usize> {
    data.windows(4).position(|window| window == b"\r\n\r\n")
}

/// 快速构造测试用 `FileObject`。
///
/// 创建具有指定 ID 的文件对象，其他字段使用合理默认值。
pub fn file_object(id: &str) -> FileObject {
    serde_json::from_value(json!({
        "id": id,
        "object": "file",
        "filename": format!("{id}.jsonl"),
        "purpose": "fine-tune"
    }))
    .unwrap()
}
