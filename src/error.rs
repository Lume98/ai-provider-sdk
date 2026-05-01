//! SDK 错误模型。统一承载配置错误、网络错误、超时错误与 API 状态错误。

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiErrorBody {
    pub message: String,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Error)]
/// SDK 统一错误类型。
///
/// 设计要点：
/// - `ApiStatus` 保留状态码、请求 ID 与原始错误体，便于上层诊断。
/// - 网络层异常与协议层异常分离，避免错误语义混淆。
pub enum Error {
    #[error("{message}")]
    ApiStatus {
        message: String,
        status: StatusCode,
        request_id: Option<String>,
        body: Option<Value>,
    },
    #[error("request timed out")]
    Timeout,
    #[error("connection error: {0}")]
    Connection(String),
    #[error("configuration error: {0}")]
    Config(String),
    #[error("invalid URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("invalid header value: {0}")]
    HeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SSE stream error: {0}")]
    Stream(String),
}

impl Error {
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

fn extract_error_message(body: &Value) -> Option<String> {
    body.get("error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
        .or_else(|| body.get("message").and_then(Value::as_str))
        .map(ToOwned::to_owned)
}
