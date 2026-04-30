pub mod openai;
pub mod sse;

pub use openai::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid header: {0}")]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),
    #[error("missing environment variable: {0}")]
    MissingEnv(&'static str),
    #[error("api error ({status}, {code}): {message}")]
    Api {
        status: u16,
        code: String,
        message: String,
        error_type: Option<String>,
        param: Option<String>,
    },
    #[error("serialization: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("stream protocol: {0}")]
    StreamProtocol(String),
    #[error("multipart/file: {0}")]
    Multipart(String),
    #[error("webhook verification failed: {0}")]
    WebhookVerification(String),
    #[error("unsupported operation: {0}")]
    Unsupported(String),
}

pub(crate) fn parse_api_error(status: u16, body: &str) -> Error {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body)
        && let Some(err) = v.get("error").and_then(serde_json::Value::as_object)
    {
        let code = err
            .get("code")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let message = err
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(body)
            .to_string();
        let error_type = err
            .get("type")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned);
        let param = err
            .get("param")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned);
        return Error::Api {
            status,
            code,
            message,
            error_type,
            param,
        };
    }
    Error::Api {
        status,
        code: "unknown".into(),
        message: body.to_string(),
        error_type: None,
        param: None,
    }
}
