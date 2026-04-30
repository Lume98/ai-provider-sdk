pub mod openai;
pub mod sse;

pub use openai::*;
use std::sync::Once;

static LOG_INIT: Once = Once::new();
const LOG_PREFIX: &str = "[vendor-ai-sdk]";

pub fn init_default_logger() {
    LOG_INIT.call_once(|| {
        let mut builder = env_logger::Builder::from_default_env();
        if std::env::var_os("RUST_LOG").is_none() {
            builder.filter_level(log::LevelFilter::Info);
        }
        builder.format(|buf, record| {
            use std::io::Write;
            let level = record.level().to_string();
            let styled_level = match record.level() {
                log::Level::Error => format!("\x1b[31m{level}\x1b[0m"),
                log::Level::Warn => format!("\x1b[33m{level}\x1b[0m"),
                log::Level::Info => format!("\x1b[32m{level}\x1b[0m"),
                log::Level::Debug => format!("\x1b[34m{level}\x1b[0m"),
                log::Level::Trace => format!("\x1b[35m{level}\x1b[0m"),
            };
            writeln!(
                buf,
                "{LOG_PREFIX} {} {}",
                styled_level,
                record.args()
            )
        });
        builder.write_style(env_logger::WriteStyle::Auto);
        builder.init();
    });
}

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
