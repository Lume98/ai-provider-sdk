use std::collections::BTreeMap;

use ai_provider::{AiError, Headers, ProviderOptions};
use bytes::Bytes;
use futures::{Stream, StreamExt};
use reqwest::multipart::Form;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio_stream::wrappers::ReceiverStream;

pub fn load_api_key(
    explicit: Option<String>,
    env_var: &str,
    provider_name: &str,
) -> Result<String, AiError> {
    match explicit.or_else(|| std::env::var(env_var).ok()) {
        Some(key) if !key.trim().is_empty() => Ok(key),
        _ => Err(AiError::Authentication(format!(
            "{provider_name} API key is missing; set {env_var}",
        ))),
    }
}

pub fn merge_headers(base: Headers, override_headers: Option<Headers>) -> Headers {
    let mut headers = base;
    if let Some(override_headers) = override_headers {
        for (key, value) in override_headers {
            headers.insert(key, value);
        }
    }
    headers
}

pub fn user_agent() -> String {
    format!("ai-rust-provider/{}", env!("CARGO_PKG_VERSION"))
}

pub async fn post_json_to_api<T: DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    headers: &Headers,
    body: &Value,
) -> Result<T, AiError> {
    let request = apply_headers(client.post(url), headers).json(body);
    let response = request.send().await.map_err(|error| AiError::ApiCall {
        message: error.to_string(),
        status: None,
        response_body: None,
    })?;

    handle_json_response(response).await
}

pub async fn post_form_to_api<T: DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    headers: &Headers,
    form: Form,
) -> Result<T, AiError> {
    let request = apply_headers(client.post(url), headers).multipart(form);
    let response = request.send().await.map_err(|error| AiError::ApiCall {
        message: error.to_string(),
        status: None,
        response_body: None,
    })?;

    handle_json_response(response).await
}

pub async fn post_form_to_api_bytes(
    client: &reqwest::Client,
    url: &str,
    headers: &Headers,
    form: Form,
) -> Result<(Vec<u8>, Headers), AiError> {
    let response = apply_headers(client.post(url), headers)
        .multipart(form)
        .send()
        .await
        .map_err(|error| AiError::ApiCall {
            message: error.to_string(),
            status: None,
            response_body: None,
        })?;

    handle_bytes_response(response).await
}

pub async fn post_json_to_api_bytes(
    client: &reqwest::Client,
    url: &str,
    headers: &Headers,
    body: &Value,
) -> Result<(Vec<u8>, Headers), AiError> {
    let response = apply_headers(client.post(url), headers)
        .json(body)
        .send()
        .await
        .map_err(|error| AiError::ApiCall {
            message: error.to_string(),
            status: None,
            response_body: None,
        })?;

    handle_bytes_response(response).await
}

pub async fn post_json_to_api_stream(
    client: &reqwest::Client,
    url: &str,
    headers: &Headers,
    body: &Value,
) -> Result<impl Stream<Item = Result<SseEvent, AiError>> + Send + 'static, AiError> {
    let response = apply_headers(client.post(url), headers)
        .json(body)
        .send()
        .await
        .map_err(|error| AiError::ApiCall {
            message: error.to_string(),
            status: None,
            response_body: None,
        })?;

    if !response.status().is_success() {
        return Err(failed_response(response).await);
    }

    Ok(parse_sse_stream(response.bytes_stream()))
}

pub fn parse_provider_options<T: DeserializeOwned>(
    provider_options: Option<&ProviderOptions>,
    provider_name: &str,
) -> Result<Option<T>, AiError> {
    let Some(options) = provider_options else {
        return Ok(None);
    };

    let Some(value) = options.get(provider_name) else {
        return Ok(None);
    };

    serde_json::from_value(value.clone())
        .map(Some)
        .map_err(|error| {
            AiError::InvalidArgument(format!("invalid {provider_name} provider options: {error}",))
        })
}

pub fn validate_json_schema(schema: &Value) -> Result<(), AiError> {
    match schema {
        Value::Object(_) | Value::Bool(_) => Ok(()),
        _ => Err(AiError::InvalidArgument(
            "JSON schema must be an object or boolean".to_string(),
        )),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SseEvent {
    pub event: Option<String>,
    pub data: String,
}

pub fn parse_sse_stream<S, E>(stream: S) -> impl Stream<Item = Result<SseEvent, AiError>> + Send
where
    S: Stream<Item = Result<Bytes, E>> + Send + 'static,
    E: std::fmt::Display + Send + 'static,
{
    let (sender, receiver) = tokio::sync::mpsc::channel(16);

    tokio::spawn(async move {
        let mut stream = Box::pin(stream);
        let mut buffer = String::new();
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    buffer.push_str(&String::from_utf8_lossy(&bytes));
                    while let Some(index) = buffer.find("\n\n") {
                        let frame = buffer[..index].to_string();
                        buffer = buffer[index + 2..].to_string();
                        if let Some(event) = parse_sse_frame(&frame) {
                            if sender.send(Ok(event)).await.is_err() {
                                return;
                            }
                        }
                    }
                }
                Err(error) => {
                    let _ = sender
                        .send(Err(AiError::ApiCall {
                            message: error.to_string(),
                            status: None,
                            response_body: None,
                        }))
                        .await;
                    return;
                }
            }
        }
    });

    ReceiverStream::new(receiver)
}

pub fn parse_sse_frame(frame: &str) -> Option<SseEvent> {
    let mut event = None;
    let mut data = Vec::new();

    for line in frame.lines() {
        if let Some(value) = line.strip_prefix("event:") {
            event = Some(value.trim_start().to_string());
        } else if let Some(value) = line.strip_prefix("data:") {
            data.push(value.trim_start().to_string());
        }
    }

    if data.is_empty() {
        None
    } else {
        Some(SseEvent {
            event,
            data: data.join("\n"),
        })
    }
}

fn apply_headers(builder: reqwest::RequestBuilder, headers: &Headers) -> reqwest::RequestBuilder {
    headers
        .iter()
        .fold(builder, |builder, (key, value)| builder.header(key, value))
}

async fn handle_json_response<T: DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, AiError> {
    if !response.status().is_success() {
        return Err(failed_response(response).await);
    }

    response.json::<T>().await.map_err(|error| {
        AiError::ResponseParsing(format!("failed to parse JSON response: {error}"))
    })
}

async fn handle_bytes_response(response: reqwest::Response) -> Result<(Vec<u8>, Headers), AiError> {
    if !response.status().is_success() {
        return Err(failed_response(response).await);
    }

    let headers = response_headers(response.headers());
    let body = response.bytes().await.map_err(|error| AiError::ApiCall {
        message: error.to_string(),
        status: None,
        response_body: None,
    })?;
    Ok((body.to_vec(), headers))
}

async fn failed_response(response: reqwest::Response) -> AiError {
    let status = response.status().as_u16();
    let body = response.text().await.unwrap_or_default();
    AiError::ApiCall {
        message: format!("provider API returned HTTP {status}"),
        status: Some(status),
        response_body: Some(body),
    }
}

fn response_headers(headers: &reqwest::header::HeaderMap) -> Headers {
    let mut output = BTreeMap::new();
    for (key, value) in headers {
        if let Ok(value) = value.to_str() {
            output.insert(key.to_string(), value.to_string());
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sse_frame() {
        assert_eq!(
            parse_sse_frame("event: response.output_text.delta\ndata: {\"delta\":\"hi\"}"),
            Some(SseEvent {
                event: Some("response.output_text.delta".to_string()),
                data: "{\"delta\":\"hi\"}".to_string(),
            })
        );
    }

    #[test]
    fn ignores_sse_comments_without_data() {
        assert_eq!(parse_sse_frame(": keep-alive"), None);
    }
}
