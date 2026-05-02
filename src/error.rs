//! SDK 错误模型。
//!
//! 统一承载配置错误、网络错误、超时错误与 API 状态错误。
//! 所有公开 API 返回的 `Result<T>` 均使用此模块定义的 [`Error`] 类型。
//!
//! ## 错误分类
//!
//! | 变体           | 触发场景                            |
//! |---------------|-----------------------------------|
//! | `ApiStatus`   | API 返回非 2xx 状态码              |
//! | `Timeout`     | 请求超过配置的超时时间              |
//! | `Connection`  | TCP 连接失败、DNS 解析失败等        |
//! | `Config`      | 参数校验失败（空 ID、互斥选项等）    |
//! | `Url`         | URL 解析/拼接失败                   |
//! | `HeaderValue` | HTTP 头值包含非法字节               |
//! | `Json`        | 请求序列化或响应反序列化失败         |
//! | `Io`          | 文件读写失败                        |
//! | `Stream`      | SSE 流解码错误或流中包含错误事件     |

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// SDK 统一 `Result` 别名。
pub type Result<T> = std::result::Result<T, Error>;

/// API 错误响应体结构。
///
/// 从服务端返回的 JSON 中解析 `message` 字段，其余字段保留到 `extra` 中。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiErrorBody {
    /// 服务端返回的人类可读错误消息。
    pub message: String,
    /// 未知字段的前向兼容保留（如 `type`、`code` 等）。
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Error)]
/// SDK 统一错误类型。
///
/// 设计要点：
/// - `ApiStatus` 保留状态码、请求 ID 与原始错误体，便于上层诊断和日志关联。
/// - 网络层异常与协议层异常分离，避免错误语义混淆。
/// - 所有变体均实现 `std::error::Error`，可通过 `#[error(...)]` 获取可读描述。
pub enum Error {
    /// API 返回非 2xx 状态码。
    ///
    /// `message` 从响应体中提取（优先取 `error.message`，回退到顶层 `message`）。
    /// `request_id` 来自 `x-request-id` 响应头，可用于向 OpenAI 提交排障请求。
    #[error("{message}")]
    ApiStatus {
        message: String,
        status: StatusCode,
        request_id: Option<String>,
        body: Option<Value>,
    },

    /// 请求超时（含连接超时和读取超时）。
    #[error("request timed out")]
    Timeout,

    /// 网络/连接层错误（DNS 解析失败、TCP 连接被拒、TLS 握手失败等）。
    #[error("connection error: {0}")]
    Connection(String),

    /// 配置/参数校验错误（互斥选项、空 ID、非法 MIME 等）。
    #[error("configuration error: {0}")]
    Config(String),

    /// URL 解析或拼接失败。
    #[error("invalid URL: {0}")]
    Url(#[from] url::ParseError),

    /// HTTP 头值包含非法字节（如非 ASCII 可见字符）。
    #[error("invalid header value: {0}")]
    HeaderValue(#[from] http::header::InvalidHeaderValue),

    /// JSON 序列化/反序列化失败。
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),

    /// I/O 错误（文件读写等）。
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// SSE 流解码错误或流中嵌入了错误事件。
    #[error("SSE stream error: {0}")]
    Stream(String),
}

impl Error {
    /// 从 HTTP 状态码、请求头和响应体构造 `ApiStatus` 错误。
    ///
    /// 消息提取优先级：
    /// 1. `body.error.message`（OpenAI 标准错误格式）
    /// 2. `body.message`（简化错误格式）
    /// 3. 回退为 `"OpenAI API returned status {status}"`
    pub fn api_status(status: StatusCode, request_id: Option<String>, body: Option<Value>) -> Self {
        let message = body
            .as_ref()
            .and_then(extract_error_message)
            .unwrap_or_else(|| format!("OpenAI API returned status {status}"));

        Self::ApiStatus {
            message,
            status,
            request_id,
            body,
        }
    }
}

/// 从 API 错误响应体中提取人类可读错误消息。
///
/// 支持两种格式：
/// - `{"error": {"message": "..."}}`（OpenAI 标准格式）
/// - `{"message": "..."}`（简化格式）
fn extract_error_message(body: &Value) -> Option<String> {
    body.get("error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
        .or_else(|| body.get("message").and_then(Value::as_str))
        .map(ToOwned::to_owned)
}
