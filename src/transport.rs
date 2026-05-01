use std::collections::HashMap;
use std::time::Duration;

use bytes::Bytes;
use http::Method;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use reqwest::multipart::{Form, Part};
use reqwest::{RequestBuilder, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::time::sleep;
use url::Url;
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::request_options::RequestOptions;
use crate::streaming::SseStream;

const INITIAL_RETRY_DELAY: Duration = Duration::from_millis(500);
const MAX_RETRY_DELAY: Duration = Duration::from_secs(8);

#[derive(Clone)]
pub(crate) struct Transport {
    http: reqwest::Client,
    base_url: Url,
    default_headers: HeaderMap,
    default_query: HashMap<String, String>,
    max_retries: u32,
}

impl Transport {
    pub(crate) fn new(
        http: reqwest::Client,
        base_url: Url,
        default_headers: HeaderMap,
        default_query: HashMap<String, String>,
        max_retries: u32,
    ) -> Self {
        Self {
            http,
            base_url,
            default_headers,
            default_query,
            max_retries,
        }
    }

    pub(crate) async fn post_json<T: DeserializeOwned>(
        &self,
        path: &str,
        body: Value,
        options: RequestOptions,
    ) -> Result<T> {
        let response = self.send(Method::POST, path, Some(body), options).await?;
        parse_response(response).await
    }

    pub(crate) async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        options: RequestOptions,
    ) -> Result<T> {
        let response = self.send(Method::GET, path, None, options).await?;
        parse_response(response).await
    }

    pub(crate) async fn delete_json<T: DeserializeOwned>(
        &self,
        path: &str,
        options: RequestOptions,
    ) -> Result<T> {
        let response = self.send(Method::DELETE, path, None, options).await?;
        parse_response(response).await
    }

    pub(crate) async fn get_bytes(&self, path: &str, options: RequestOptions) -> Result<Bytes> {
        let response = self.send(Method::GET, path, None, options).await?;
        parse_bytes_response(response).await
    }

    pub(crate) async fn post_multipart_json<T: DeserializeOwned>(
        &self,
        path: &str,
        form: MultipartFormData,
        options: RequestOptions,
    ) -> Result<T> {
        let response = self
            .send_multipart(Method::POST, path, form, options)
            .await?;
        parse_response(response).await
    }

    pub(crate) async fn post_stream(
        &self,
        path: &str,
        body: Value,
        options: RequestOptions,
    ) -> Result<SseStream> {
        let response = self.send(Method::POST, path, Some(body), options).await?;
        let status = response.status();
        let request_id = request_id(response.headers());

        if !status.is_success() {
            let bytes = response.bytes().await.map_err(map_reqwest_error)?;
            let body = parse_json_body(&bytes);
            return Err(Error::api_status(status, request_id, body));
        }

        Ok(SseStream::new(response))
    }

    async fn send(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
        mut options: RequestOptions,
    ) -> Result<reqwest::Response> {
        let url = self.url(path, &options)?;
        let body = body.map(|body| merge_extra_body(body, options.extra_body.take()));
        let idempotency_key =
            (method != Method::GET).then(|| format!("stainless-rust-retry-{}", Uuid::new_v4()));
        let mut attempt = 0;

        loop {
            let request = self.request_builder(
                method.clone(),
                url.clone(),
                body.as_ref(),
                &options,
                idempotency_key.as_deref(),
            )?;
            let result = request.send().await;

            match result {
                Ok(response) => {
                    if attempt < self.max_retries && should_retry(&response) {
                        let delay = retry_delay(attempt, Some(response.headers()));
                        attempt += 1;
                        sleep(delay).await;
                        continue;
                    }
                    return Ok(response);
                }
                Err(err) => {
                    if attempt < self.max_retries && (err.is_timeout() || err.is_connect()) {
                        let delay = retry_delay(attempt, None);
                        attempt += 1;
                        sleep(delay).await;
                        continue;
                    }
                    return Err(map_reqwest_error(err));
                }
            }
        }
    }

    async fn send_multipart(
        &self,
        method: Method,
        path: &str,
        form: MultipartFormData,
        options: RequestOptions,
    ) -> Result<reqwest::Response> {
        let url = self.url(path, &options)?;
        let idempotency_key =
            (method != Method::GET).then(|| format!("stainless-rust-retry-{}", Uuid::new_v4()));
        let mut attempt = 0;

        loop {
            let request = self.request_builder_multipart(
                method.clone(),
                url.clone(),
                form.clone().into_form()?,
                &options,
                idempotency_key.as_deref(),
            )?;
            let result = request.send().await;

            match result {
                Ok(response) => {
                    if attempt < self.max_retries && should_retry(&response) {
                        let delay = retry_delay(attempt, Some(response.headers()));
                        attempt += 1;
                        sleep(delay).await;
                        continue;
                    }
                    return Ok(response);
                }
                Err(err) => {
                    if attempt < self.max_retries && (err.is_timeout() || err.is_connect()) {
                        let delay = retry_delay(attempt, None);
                        attempt += 1;
                        sleep(delay).await;
                        continue;
                    }
                    return Err(map_reqwest_error(err));
                }
            }
        }
    }

    fn request_builder(
        &self,
        method: Method,
        url: Url,
        body: Option<&Value>,
        options: &RequestOptions,
        idempotency_key: Option<&str>,
    ) -> Result<RequestBuilder> {
        let mut headers = self.default_headers.clone();
        if let Some(idempotency_key) = idempotency_key {
            headers.insert("idempotency-key", HeaderValue::from_str(idempotency_key)?);
        }

        for (key, value) in &options.extra_headers {
            let name = HeaderName::from_bytes(key.as_bytes())
                .map_err(|err| Error::Config(format!("invalid header name `{key}`: {err}")))?;
            headers.insert(name, HeaderValue::from_str(value)?);
        }

        let mut builder = self.http.request(method, url).headers(headers);
        if let Some(body) = body {
            builder = builder.json(body);
        }
        if let Some(timeout) = options.timeout {
            builder = builder.timeout(timeout);
        }
        Ok(builder)
    }

    fn request_builder_multipart(
        &self,
        method: Method,
        url: Url,
        form: Form,
        options: &RequestOptions,
        idempotency_key: Option<&str>,
    ) -> Result<RequestBuilder> {
        let mut headers = self.default_headers.clone();
        headers.remove(CONTENT_TYPE);

        if let Some(idempotency_key) = idempotency_key {
            headers.insert("idempotency-key", HeaderValue::from_str(idempotency_key)?);
        }

        for (key, value) in &options.extra_headers {
            let name = HeaderName::from_bytes(key.as_bytes())
                .map_err(|err| Error::Config(format!("invalid header name `{key}`: {err}")))?;
            headers.insert(name, HeaderValue::from_str(value)?);
        }

        let mut builder = self
            .http
            .request(method, url)
            .headers(headers)
            .multipart(form);
        if let Some(timeout) = options.timeout {
            builder = builder.timeout(timeout);
        }
        Ok(builder)
    }

    fn url(&self, path: &str, options: &RequestOptions) -> Result<Url> {
        let path = path.trim_start_matches('/');
        let mut url = self.base_url.join(path)?;
        {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in &self.default_query {
                pairs.append_pair(key, value);
            }
            for (key, value) in &options.extra_query {
                pairs.append_pair(key, value);
            }
        }
        Ok(url)
    }
}

#[derive(Clone)]
pub(crate) struct MultipartFormData {
    fields: Vec<(String, String)>,
    files: Vec<MultipartFile>,
}

impl MultipartFormData {
    pub(crate) fn new() -> Self {
        Self {
            fields: Vec::new(),
            files: Vec::new(),
        }
    }

    pub(crate) fn text(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push((name.into(), value.into()));
        self
    }

    pub(crate) fn file(mut self, file: MultipartFile) -> Self {
        self.files.push(file);
        self
    }

    fn into_form(self) -> Result<Form> {
        let mut form = Form::new();
        for (name, value) in self.fields {
            form = form.text(name, value);
        }

        for file in self.files {
            let length = file.bytes.len() as u64;
            let mut part = Part::stream_with_length(file.bytes, length).file_name(file.file_name);
            if let Some(mime_type) = file.mime_type {
                part = part.mime_str(&mime_type).map_err(|err| {
                    Error::Config(format!("invalid MIME type `{mime_type}`: {err}"))
                })?;
            }
            form = form.part(file.field_name, part);
        }

        Ok(form)
    }
}

#[derive(Clone)]
pub(crate) struct MultipartFile {
    pub(crate) field_name: String,
    pub(crate) file_name: String,
    pub(crate) bytes: Bytes,
    pub(crate) mime_type: Option<String>,
}

async fn parse_response<T: DeserializeOwned>(response: reqwest::Response) -> Result<T> {
    let status = response.status();
    let request_id = request_id(response.headers());
    let bytes = response.bytes().await.map_err(map_reqwest_error)?;

    if !status.is_success() {
        let body = parse_json_body(&bytes);
        return Err(Error::api_status(status, request_id, body));
    }

    serde_json::from_slice(&bytes).map_err(Error::from)
}

async fn parse_bytes_response(response: reqwest::Response) -> Result<Bytes> {
    let status = response.status();
    let request_id = request_id(response.headers());
    let bytes = response.bytes().await.map_err(map_reqwest_error)?;

    if !status.is_success() {
        let body = parse_json_body(&bytes);
        return Err(Error::api_status(status, request_id, body));
    }

    Ok(bytes)
}

fn should_retry(response: &reqwest::Response) -> bool {
    match response
        .headers()
        .get("x-should-retry")
        .and_then(|v| v.to_str().ok())
    {
        Some("true") => return true,
        Some("false") => return false,
        _ => {}
    }

    matches!(
        response.status(),
        StatusCode::REQUEST_TIMEOUT | StatusCode::CONFLICT | StatusCode::TOO_MANY_REQUESTS
    ) || response.status().is_server_error()
}

fn retry_delay(attempt: u32, headers: Option<&HeaderMap>) -> Duration {
    if let Some(delay) = headers.and_then(parse_retry_after) {
        if delay > Duration::ZERO && delay <= Duration::from_secs(60) {
            return delay;
        }
    }

    let factor = 2_u32.saturating_pow(attempt.min(10));
    (INITIAL_RETRY_DELAY * factor).min(MAX_RETRY_DELAY)
}

fn parse_retry_after(headers: &HeaderMap) -> Option<Duration> {
    headers
        .get("retry-after-ms")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
        .or_else(|| {
            headers
                .get("retry-after")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<u64>().ok())
                .map(Duration::from_secs)
        })
}

fn request_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

fn parse_json_body(bytes: &[u8]) -> Option<Value> {
    serde_json::from_slice(bytes).ok()
}

fn merge_extra_body(mut body: Value, extra_body: Option<Value>) -> Value {
    let Some(extra_body) = extra_body else {
        return body;
    };

    match (&mut body, extra_body) {
        (Value::Object(body), Value::Object(extra)) => {
            for (key, value) in extra {
                body.insert(key, value);
            }
            Value::Object(std::mem::take(body))
        }
        (_, extra) => extra,
    }
}

fn map_reqwest_error(err: reqwest::Error) -> Error {
    if err.is_timeout() {
        Error::Timeout
    } else {
        Error::Connection(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extra_body_overrides_body_fields() {
        let body = serde_json::json!({"model": "a", "input": "x"});
        let extra = serde_json::json!({"model": "b", "metadata": {"k": "v"}});

        assert_eq!(
            merge_extra_body(body, Some(extra)),
            serde_json::json!({"model": "b", "input": "x", "metadata": {"k": "v"}})
        );
    }

    #[test]
    fn retries_target_python_status_set() {
        let statuses = [
            StatusCode::REQUEST_TIMEOUT,
            StatusCode::CONFLICT,
            StatusCode::TOO_MANY_REQUESTS,
            StatusCode::INTERNAL_SERVER_ERROR,
        ];

        for status in statuses {
            let response = http::Response::builder().status(status).body("").unwrap();
            let response = reqwest::Response::from(response);
            assert!(should_retry(&response));
        }
    }

    #[test]
    fn parses_retry_after_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("retry-after-ms", HeaderValue::from_static("250"));
        assert_eq!(
            parse_retry_after(&headers),
            Some(Duration::from_millis(250))
        );
    }
}
