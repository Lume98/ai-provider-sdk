//! HTTP 传输层。
//!
//! 负责所有与 HTTP 协议相关的底层工作：
//! - URL 组装（base URL + 路径 + 查询参数）
//! - 请求发送（JSON body、multipart/form-data）
//! - 重试策略（指数退避 + 服务端重试头）
//! - 响应解析（JSON 反序列化、二进制流、SSE 流）
//!
//! ## 重试机制
//!
//! 传输层对以下情况自动重试（最多 `max_retries` 次）：
//!
//! | 触发条件                                  | 重试行为                |
//! |-----------------------------------------|-----------------------|
//! | HTTP 408 / 409 / 429 / 5xx              | 指数退避或服务端重试间隔   |
//! | `x-should-retry: true` 响应头            | 立即重试                |
//! | 连接错误 / 超时                           | 指数退避                |
//!
//! 每次非 GET 请求会生成唯一的 `idempotency-key`，重试时保持不变，
//! 确保服务端可安全处理重复请求。

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

/// 重试初始退避时间（500ms），后续每次翻倍直到达到上限。
const INITIAL_RETRY_DELAY: Duration = Duration::from_millis(500);

/// 重试最大退避时间上限（8s）。
const MAX_RETRY_DELAY: Duration = Duration::from_secs(8);

#[derive(Clone)]
/// 传输层核心对象。
///
/// 状态边界：
/// - 持有全局默认头与查询参数，每次请求时克隆使用。
/// - 不持有业务状态，`Clone` 后可安全并发复用。
/// - 内部 `reqwest::Client` 天然支持连接池复用。
pub(crate) struct Transport {
    /// 底层 HTTP 客户端（自带连接池）。
    http: reqwest::Client,
    /// 归一化后的 base URL（以 `/` 结尾）。
    base_url: Url,
    /// 全局默认 HTTP 头（鉴权、组织/项目标识等）。
    default_headers: HeaderMap,
    /// 全局默认查询参数。
    default_query: HashMap<String, String>,
    /// 最大重试次数。
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

    /// 发送 POST JSON 请求并按目标类型反序列化响应体。
    ///
    /// 适用于 Chat Completions、Embeddings、Moderations 等标准 POST 接口。
    pub(crate) async fn post_json<T: DeserializeOwned>(
        &self,
        path: &str,
        body: Value,
        options: RequestOptions,
    ) -> Result<T> {
        let response = self.send(Method::POST, path, Some(body), options).await?;
        parse_response(response).await
    }

    /// 发送 GET 请求并按目标类型反序列化响应体。
    ///
    /// 适用于 Models 列表、Files 列表等查询接口。
    pub(crate) async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        options: RequestOptions,
    ) -> Result<T> {
        let response = self.send(Method::GET, path, None, options).await?;
        parse_response(response).await
    }

    /// 发送 DELETE 请求并按目标类型反序列化响应体。
    ///
    /// 适用于 Files 删除等接口。
    pub(crate) async fn delete_json<T: DeserializeOwned>(
        &self,
        path: &str,
        options: RequestOptions,
    ) -> Result<T> {
        let response = self.send(Method::DELETE, path, None, options).await?;
        parse_response(response).await
    }

    /// 发送 GET 请求并返回原始二进制响应体。
    ///
    /// 适用于 Files 内容下载等二进制接口。
    pub(crate) async fn get_bytes(&self, path: &str, options: RequestOptions) -> Result<Bytes> {
        let response = self.send(Method::GET, path, None, options).await?;
        parse_bytes_response(response).await
    }

    /// 发送 multipart/form-data POST 请求并按目标类型反序列化响应体。
    ///
    /// 适用于文件上传接口（`/files`），需要发送二进制文件内容。
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

    /// 发送流式 POST 请求并返回 SSE 包装流。
    ///
    /// 与 `post_json` 的差异是：成功后不立刻消费 body，
    /// 而是把 response 交给流解码层按 SSE 协议逐事件产出。
    ///
    /// 错误处理：如果服务端返回非 2xx 状态码（包括流式请求被拒绝的场景），
    /// 会立即读取完整错误体并返回 `ApiStatus` 错误。
    pub(crate) async fn post_stream(
        &self,
        path: &str,
        body: Value,
        options: RequestOptions,
    ) -> Result<SseStream> {
        let response = self.send(Method::POST, path, Some(body), options).await?;
        let status = response.status();
        let request_id = request_id(response.headers());

        // 流式请求的错误需要在返回 SseStream 之前处理
        if !status.is_success() {
            let bytes = response.bytes().await.map_err(map_reqwest_error)?;
            let body = parse_json_body(&bytes);
            return Err(Error::api_status(status, request_id, body));
        }

        Ok(SseStream::new(response))
    }

    /// 统一发送 JSON 请求，内置重试与幂等键策略。
    ///
    /// 重试流程：
    /// 1. 为非 GET 请求生成唯一 `idempotency-key`。
    /// 2. 发送请求，根据状态码或连接错误判断是否重试。
    /// 3. 重试时使用指数退避或服务端 `retry-after` 头指定的间隔。
    /// 4. 重试时保持相同的 `idempotency-key`，确保幂等性。
    async fn send(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
        mut options: RequestOptions,
    ) -> Result<reqwest::Response> {
        let url = self.url(path, &options)?;
        // 合并 extra_body 到原始请求体
        let body = body.map(|body| merge_extra_body(body, options.extra_body.take()));
        // 非 GET 请求生成幂等键（整个重试周期保持不变）
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
                    // 可重试的状态码，且未超过最大重试次数
                    if attempt < self.max_retries && should_retry(&response) {
                        let delay = retry_delay(attempt, Some(response.headers()));
                        attempt += 1;
                        sleep(delay).await;
                        continue;
                    }
                    return Ok(response);
                }
                Err(err) => {
                    // 连接错误或超时，且未超过最大重试次数
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

    /// 发送 multipart/form-data 请求，内置重试策略。
    ///
    /// 逻辑与 `send` 相同，区别在于请求体为 multipart form 而非 JSON。
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
    ///
    /// 头优先级：请求级 `extra_headers` > 全局 `default_headers`。
    fn request_builder(
        &self,
        method: Method,
        url: Url,
        body: Option<&Value>,
        options: &RequestOptions,
        idempotency_key: Option<&str>,
    ) -> Result<RequestBuilder> {
        let mut headers = self.default_headers.clone();

        // 追加幂等键（重试时保持不变）
        if let Some(idempotency_key) = idempotency_key {
            headers.insert("idempotency-key", HeaderValue::from_str(idempotency_key)?);
        }

        // 合并请求级覆盖头
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
    /// 注意：multipart 场景必须移除手动设置的 `Content-Type: application/json`，
    /// 由 `reqwest` 自动填充带 boundary 的 `multipart/form-data` content-type。
    fn request_builder_multipart(
        &self,
        method: Method,
        url: Url,
        form: Form,
        options: &RequestOptions,
        idempotency_key: Option<&str>,
    ) -> Result<RequestBuilder> {
        let mut headers = self.default_headers.clone();
        // 移除 JSON content-type，让 reqwest 自动设置 multipart boundary
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

    /// 组装完整请求 URL。
    ///
    /// 将 base URL + 路径片段 + 全局默认查询参数 + 请求级查询参数合并为最终 URL。
    fn url(&self, path: &str, options: &RequestOptions) -> Result<Url> {
        // 去掉路径前导 `/`，因为 base URL 已以 `/` 结尾
        let path = path.trim_start_matches('/');
        let mut url = self.base_url.join(path)?;
        {
            let mut pairs = url.query_pairs_mut();
            // 先写全局默认查询参数
            for (key, value) in &self.default_query {
                pairs.append_pair(key, value);
            }
            // 再写请求级查询参数
            for (key, value) in &options.extra_query {
                pairs.append_pair(key, value);
            }
        }
        Ok(url)
    }
}

#[derive(Clone)]
/// Multipart 表单数据容器。
///
/// 区分文本字段和文件字段，最终转换为 `reqwest::multipart::Form` 发送。
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

    /// 追加文本字段（builder 模式）。
    pub(crate) fn text(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push((name.into(), value.into()));
        self
    }

    /// 追加文件字段（builder 模式）。
    pub(crate) fn file(mut self, file: MultipartFile) -> Self {
        self.files.push(file);
        self
    }

    /// 转换为 `reqwest::multipart::Form`。
    ///
    /// 边界条件：若 MIME 不合法，返回配置错误，避免请求发出后才失败。
    fn into_form(self) -> Result<Form> {
        let mut form = Form::new();

        // 追加文本字段
        for (name, value) in self.fields {
            form = form.text(name, value);
        }

        // 追加文件字段
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

/// Multipart 文件字段描述。
#[derive(Clone)]
pub(crate) struct MultipartFile {
    /// multipart 字段名（如 `file`）。
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
///
/// 用于文件内容下载等非 JSON 响应场景。
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
/// 判定优先级：
/// 1. `x-should-retry: true` → 立即重试。
/// 2. `x-should-retry: false` → 不重试。
/// 3. 状态码为 408 / 409 / 429 / 5xx → 重试。
///
/// 隐性耦合：该集合需与官方 SDK 语义保持一致，避免跨语言行为偏差。
fn should_retry(response: &reqwest::Response) -> bool {
    // 优先检查服务端显式重试指令
    match response
        .headers()
        .get("x-should-retry")
        .and_then(|v| v.to_str().ok())
    {
        Some("true") => return true,
        Some("false") => return false,
        _ => {}
    }

    // 根据状态码判断
    matches!(
        response.status(),
        StatusCode::REQUEST_TIMEOUT | StatusCode::CONFLICT | StatusCode::TOO_MANY_REQUESTS
    ) || response.status().is_server_error()
}

/// 计算重试延迟。
///
/// 优先使用服务端通过响应头指定的延迟（`retry-after-ms` 或 `retry-after`），
/// 否则使用指数退避策略。
///
/// 退避公式：`INITIAL_RETRY_DELAY * 2^attempt`，上限为 `MAX_RETRY_DELAY`。
fn retry_delay(attempt: u32, headers: Option<&HeaderMap>) -> Duration {
    // 优先使用服务端指定的重试间隔
    if let Some(delay) = headers.and_then(parse_retry_after) {
        if delay > Duration::ZERO && delay <= Duration::from_secs(60) {
            return delay;
        }
    }

    // 指数退避：500ms, 1s, 2s, 4s, 8s, 8s, ...
    let factor = 2_u32.saturating_pow(attempt.min(10));
    (INITIAL_RETRY_DELAY * factor).min(MAX_RETRY_DELAY)
}

/// 从响应头中解析服务端重试延迟。
///
/// 优先级：`retry-after-ms` > `retry-after`。
/// `retry-after` 仅支持秒级整数值（不支持 HTTP 日期格式）。
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

/// 获取 OpenAI 请求链路 ID（`x-request-id` 响应头）。
///
/// 可用于错误日志关联和向 OpenAI 提交排障请求。
fn request_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

/// 尝试把错误响应体解析成 JSON。
///
/// 即使解析失败也返回 `None`，不阻断错误处理流程。
fn parse_json_body(bytes: &[u8]) -> Option<Value> {
    serde_json::from_slice(bytes).ok()
}

/// 合并 `extra_body` 到原始请求体。
///
/// 设计取舍：
/// - 当两者均为 JSON 对象时，执行浅层按键覆盖（`extra_body` 的值优先）。
/// - 非对象场景下，`extra_body` 直接替换原值。
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
///
/// 超时错误映射为 `Error::Timeout`，其余映射为 `Error::Connection`。
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
        // 确保重试状态码集合与 Python SDK 保持一致
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
