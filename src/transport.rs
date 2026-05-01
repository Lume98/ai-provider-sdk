//! HTTP 传输层。负责 URL 组装、重试策略、请求发送、响应解析与 multipart 编码。

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
/// 传输层核心对象。
///
/// 状态边界：
/// - 持有全局默认头与查询参数。
/// - 不持有业务状态，`Clone` 后可并发复用。
pub(crate) struct Transport {
    http: reqwest::Client,
    base_url: Url,
    default_headers: HeaderMap,
    default_query: HashMap<String, String>,
    max_retries: u32,
}

impl Transport {
    /// 构建传输层实例。
    ///
    /// 调用方需保证 `base_url` 已归一化为目录路径（通常以 `/` 结尾）。
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

    /// 发送 POST JSON 请求并按目标类型反序列化。
    pub(crate) async fn post_json<T: DeserializeOwned>(
        &self,
        path: &str,
        body: Value,
        options: RequestOptions,
    ) -> Result<T> {
        let response = self.send(Method::POST, path, Some(body), options).await?;
        parse_response(response).await
    }

    /// 发送 GET 请求并按目标类型反序列化。
    pub(crate) async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        options: RequestOptions,
    ) -> Result<T> {
        let response = self.send(Method::GET, path, None, options).await?;
        parse_response(response).await
    }

    /// 发送 DELETE 请求并按目标类型反序列化。
    pub(crate) async fn delete_json<T: DeserializeOwned>(
        &self,
        path: &str,
        options: RequestOptions,
    ) -> Result<T> {
        let response = self.send(Method::DELETE, path, None, options).await?;
        parse_response(response).await
    }

    /// 发送 GET 请求并返回原始二进制响应体。
    pub(crate) async fn get_bytes(&self, path: &str, options: RequestOptions) -> Result<Bytes> {
        let response = self.send(Method::GET, path, None, options).await?;
        parse_bytes_response(response).await
    }

    /// 发送 multipart/form-data 请求并按目标类型反序列化。
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

    /// 发送流式请求并返回 SSE 包装流。
    ///
    /// 与 `post_json` 的差异是：成功后不立刻消费 body，而是把 response 交给流解码层。
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

    /// 统一发送 JSON 请求，内置重试与幂等键策略。
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

    /// 构建 JSON 请求构造器，合并默认头与请求级覆盖头。
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

    /// 构建 multipart 请求构造器。
    ///
    /// 注意：multipart 场景必须移除手动 `Content-Type`，由 `reqwest` 自动填充 boundary。
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

    /// 组装完整请求 URL，并合并默认查询参数与请求级查询参数。
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
    /// 文本字段集合（name -> value）。
    fields: Vec<(String, String)>,
    /// 文件字段集合。
    files: Vec<MultipartFile>,
}

impl MultipartFormData {
    /// 创建空表单容器。
    pub(crate) fn new() -> Self {
        Self {
            fields: Vec::new(),
            files: Vec::new(),
        }
    }

    /// 追加文本字段。
    pub(crate) fn text(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push((name.into(), value.into()));
        self
    }

    /// 追加文件字段。
    pub(crate) fn file(mut self, file: MultipartFile) -> Self {
        self.files.push(file);
        self
    }

    /// 转换为 `reqwest::multipart::Form`。
    ///
    /// 边界条件：若 MIME 不合法，返回配置错误，避免请求发出后才失败。
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
    /// multipart 字段名，例如 `file`。
    pub(crate) field_name: String,
    /// 上送给服务端的文件名。
    pub(crate) file_name: String,
    /// 文件字节内容。
    pub(crate) bytes: Bytes,
    /// 可选 MIME 类型；若为空由服务端或下游自行推断。
    pub(crate) mime_type: Option<String>,
}

/// 解析 JSON 类型响应。
///
/// 失败策略：
/// - 非 2xx：构造 `ApiStatus`，保留状态码、request_id 与原始 JSON body（若可解析）。
/// - 2xx 但反序列化失败：返回 `Error::Json`。
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

/// 解析二进制响应；仅在状态成功时返回字节流。
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

/// 判断响应是否可重试。
///
/// 隐性耦合：该集合需与官方 SDK 语义保持一致，避免跨语言行为偏差。
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

/// 从响应头中解析服务端重试延迟。
///
/// 优先级：`retry-after-ms` > `retry-after`。
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

/// 获取 OpenAI 请求链路 ID，用于错误可观测性。
fn request_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

/// 尝试把错误响应体解析成 JSON。
fn parse_json_body(bytes: &[u8]) -> Option<Value> {
    serde_json::from_slice(bytes).ok()
}

/// 合并 `extra_body` 到原始请求体。
///
/// 设计取舍：当两者均为对象时执行浅层覆盖；否则用 `extra_body` 替换原值。
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

/// 将 `reqwest::Error` 映射到 SDK 错误模型。
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
