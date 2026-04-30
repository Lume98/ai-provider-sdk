use std::{
    env,
    time::{Duration, Instant},
};

use bytes::Bytes;
use futures::StreamExt;
use log::{debug, warn};
use reqwest::{
    Client, Method, Response, StatusCode,
    header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue},
    multipart,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::{Error, parse_api_error, sse::SseJsonStream};

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const USER_AGENT: &str = concat!("vendor-ai-sdk/", env!("CARGO_PKG_VERSION"));
const ANSI_RESET: &str = "\x1b[0m";
const ANSI_DIM: &str = "\x1b[2m";
const ANSI_BOLD_CYAN: &str = "\x1b[1;36m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_MAGENTA: &str = "\x1b[35m";
const ANSI_BLUE: &str = "\x1b[34m";
const ANSI_RED: &str = "\x1b[31m";

#[derive(Clone, Debug)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub base_url: String,
    pub organization: Option<String>,
    pub project: Option<String>,
    pub timeout: Option<Duration>,
    pub max_retries: u32,
    pub default_headers: HeaderMap,
    pub default_query: Vec<(String, String)>,
}

impl OpenAIConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            organization: None,
            project: None,
            timeout: None,
            max_retries: 2,
            default_headers: HeaderMap::new(),
            default_query: Vec::new(),
        }
    }

    pub fn from_env() -> Result<Self, Error> {
        let api_key =
            env::var("OPENAI_API_KEY").map_err(|_| Error::MissingEnv("OPENAI_API_KEY"))?;
        Ok(Self::new(api_key)
            .with_base_url(env::var("OPENAI_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.into()))
            .with_optional_organization(env::var("OPENAI_ORG_ID").ok())
            .with_optional_project(env::var("OPENAI_PROJECT_ID").ok()))
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn with_organization(mut self, organization: impl Into<String>) -> Self {
        self.organization = Some(organization.into());
        self
    }

    pub fn with_project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn with_default_header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.default_headers.insert(name, value);
        self
    }

    pub fn with_default_query(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_query.push((name.into(), value.into()));
        self
    }

    fn with_optional_organization(mut self, organization: Option<String>) -> Self {
        self.organization = organization;
        self
    }

    fn with_optional_project(mut self, project: Option<String>) -> Self {
        self.project = project;
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct RequestOptions {
    pub headers: HeaderMap,
    pub query: Vec<(String, String)>,
    pub idempotency_key: Option<String>,
}

impl RequestOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.headers.insert(name, value);
        self
    }

    pub fn with_query(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.push((name.into(), value.into()));
        self
    }

    pub fn with_idempotency_key(mut self, key: impl Into<String>) -> Self {
        self.idempotency_key = Some(key.into());
        self
    }
}

#[derive(Clone)]
pub(crate) struct HttpCore {
    http: Client,
    pub(crate) config: OpenAIConfig,
}

impl HttpCore {
    pub(crate) fn new(config: OpenAIConfig) -> Self {
        let mut builder = Client::builder().user_agent(USER_AGENT);
        if let Some(timeout) = config.timeout {
            builder = builder.timeout(timeout);
        }
        Self {
            http: builder.build().expect("valid reqwest client"),
            config,
        }
    }

    pub(crate) async fn json<T, R>(
        &self,
        method: Method,
        path: &str,
        query: Option<&T>,
        body: Option<&R>,
        options: RequestOptions,
    ) -> Result<Response, Error>
    where
        T: Serialize + ?Sized,
        R: Serialize + ?Sized,
    {
        let serialized_query = serialize_json_value(query)?;
        let serialized_body = serialize_json_value(body)?;
        self.print_request_trace(
            &method,
            path,
            serialized_query.as_ref(),
            serialized_body.as_ref(),
            &options,
        );

        let mut attempt = 0;
        let started_at = Instant::now();
        loop {
            let attempt_no = attempt + 1;
            debug!(
                "openai request start: method={}, path={}, attempt={}",
                method, path, attempt_no
            );
            let response = self
                .build(method.clone(), path, query, options.clone())?
                .header(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            let response = if let Some(body) = body {
                response.json(body).send().await
            } else {
                response.send().await
            };

            match response {
                Ok(response)
                    if should_retry(response.status()) && attempt < self.config.max_retries =>
                {
                    attempt += 1;
                    warn!(
                        "openai request retryable status: method={}, path={}, status={}, attempt={}, elapsed_ms={}",
                        method,
                        path,
                        response.status().as_u16(),
                        attempt_no,
                        started_at.elapsed().as_millis()
                    );
                    sleep_retry(attempt).await;
                }
                Ok(response) => {
                    debug!(
                        "openai request done: method={}, path={}, status={}, attempts={}, elapsed_ms={}",
                        method,
                        path,
                        response.status().as_u16(),
                        attempt_no,
                        started_at.elapsed().as_millis()
                    );
                    return Ok(response);
                }
                Err(err)
                    if (err.is_timeout() || err.is_connect())
                        && attempt < self.config.max_retries =>
                {
                    attempt += 1;
                    warn!(
                        "openai request retryable error: method={}, path={}, attempt={}, elapsed_ms={}, error={}",
                        method,
                        path,
                        attempt_no,
                        started_at.elapsed().as_millis(),
                        err
                    );
                    sleep_retry(attempt).await;
                }
                Err(err) => {
                    warn!(
                        "openai request failed: method={}, path={}, attempt={}, elapsed_ms={}, error={}",
                        method,
                        path,
                        attempt_no,
                        started_at.elapsed().as_millis(),
                        err
                    );
                    return Err(Error::Http(err));
                }
            }
        }
    }

    pub(crate) async fn json_value<T, R, O>(
        &self,
        method: Method,
        path: &str,
        query: Option<&T>,
        body: Option<&R>,
        options: RequestOptions,
    ) -> Result<O, Error>
    where
        T: Serialize + ?Sized,
        R: Serialize + ?Sized,
        O: DeserializeOwned,
    {
        let response = self.json(method, path, query, body, options).await?;
        parse_json_response(response).await
    }

    pub(crate) async fn bytes<T: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        query: Option<&T>,
        options: RequestOptions,
    ) -> Result<Bytes, Error> {
        let response = self
            .json::<T, ()>(method, path, query, None, options)
            .await?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await?;
            return Err(parse_api_error(status.as_u16(), &text));
        }
        Ok(response.bytes().await?)
    }

    pub(crate) async fn multipart<O: DeserializeOwned>(
        &self,
        path: &str,
        form: multipart::Form,
        options: RequestOptions,
    ) -> Result<O, Error> {
        let started_at = Instant::now();
        debug!("openai multipart request start: method=POST, path={}", path);
        let response = self
            .build(Method::POST, path, Option::<&()>::None, options)?
            .multipart(form)
            .send()
            .await?;
        debug!(
            "openai multipart request done: method=POST, path={}, status={}, elapsed_ms={}",
            path,
            response.status().as_u16(),
            started_at.elapsed().as_millis()
        );
        parse_json_response(response).await
    }

    pub(crate) async fn stream<T, R, O>(
        &self,
        method: Method,
        path: &str,
        query: Option<&T>,
        body: Option<&R>,
        options: RequestOptions,
    ) -> Result<TypedSseStream<O>, Error>
    where
        T: Serialize + ?Sized,
        R: Serialize + ?Sized,
        O: DeserializeOwned,
    {
        debug!("openai stream start: method={}, path={}", method, path);
        let response = self.json(method, path, query, body, options).await?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await?;
            warn!("openai stream rejected: status={}, path={}", status.as_u16(), path);
            return Err(parse_api_error(status.as_u16(), &text));
        }
        debug!("openai stream established: status={}, path={}", status.as_u16(), path);
        Ok(TypedSseStream::new(SseJsonStream::new(
            response.bytes_stream(),
        )))
    }

    fn build<T: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        query: Option<&T>,
        options: RequestOptions,
    ) -> Result<reqwest::RequestBuilder, Error> {
        let url = format!("{}{}", self.config.base_url.trim_end_matches('/'), path);
        let mut request = self
            .http
            .request(method, url)
            .header(AUTHORIZATION, bearer_header_value(&self.config.api_key)?);

        for (name, value) in &self.config.default_headers {
            request = request.header(name, value);
        }
        for (name, value) in &options.headers {
            request = request.header(name, value);
        }
        if let Some(organization) = &self.config.organization {
            request = request.header("OpenAI-Organization", organization);
        }
        if let Some(project) = &self.config.project {
            request = request.header("OpenAI-Project", project);
        }
        if let Some(key) = options.idempotency_key {
            request = request.header("Idempotency-Key", key);
        }
        if !self.config.default_query.is_empty() {
            request = request.query(&self.config.default_query);
        }
        if !options.query.is_empty() {
            request = request.query(&options.query);
        }
        if let Some(query) = query {
            request = request.query(query);
        }
        Ok(request)
    }
}

impl HttpCore {
    fn print_request_trace(
        &self,
        method: &Method,
        path: &str,
        query: Option<&Value>,
        body: Option<&Value>,
        options: &RequestOptions,
    ) {
        let url = format!("{}{}", self.config.base_url.trim_end_matches('/'), path);
        eprintln!(
            "{}[vendor-ai-sdk]{} {}request{} {}method={}{} {}url={}{}",
            ANSI_BOLD_CYAN,
            ANSI_RESET,
            ANSI_DIM,
            ANSI_RESET,
            ANSI_GREEN,
            method,
            ANSI_RESET,
            ANSI_YELLOW,
            url,
            ANSI_RESET
        );

        let headers = self.collect_headers_preview(options);
        eprintln!(
            "{}[vendor-ai-sdk]{} {}request{} {}headers={}{}",
            ANSI_BOLD_CYAN, ANSI_RESET, ANSI_DIM, ANSI_RESET, ANSI_BLUE, headers, ANSI_RESET
        );

        if let Some(query) = query {
            eprintln!(
                "{}[vendor-ai-sdk]{} {}request{} {}query={}{}",
                ANSI_BOLD_CYAN,
                ANSI_RESET,
                ANSI_DIM,
                ANSI_RESET,
                ANSI_MAGENTA,
                compact_json(query),
                ANSI_RESET
            );
        }
        if let Some(body) = body {
            eprintln!(
                "{}[vendor-ai-sdk]{} {}request{} {}body={}{}",
                ANSI_BOLD_CYAN,
                ANSI_RESET,
                ANSI_DIM,
                ANSI_RESET,
                ANSI_MAGENTA,
                compact_json(body),
                ANSI_RESET
            );
            if let Some(messages) = body.get("messages") {
                eprintln!(
                    "{}[vendor-ai-sdk]{} {}request{} {}messages={}{}",
                    ANSI_BOLD_CYAN,
                    ANSI_RESET,
                    ANSI_DIM,
                    ANSI_RESET,
                    ANSI_GREEN,
                    compact_json(messages),
                    ANSI_RESET
                );
            }
        }
    }

    fn collect_headers_preview(&self, options: &RequestOptions) -> String {
        let mut pairs: Vec<String> = Vec::new();
        pairs.push("authorization=Bearer ***".to_string());

        if self.config.organization.is_some() {
            pairs.push("openai-organization=<set>".to_string());
        }
        if self.config.project.is_some() {
            pairs.push("openai-project=<set>".to_string());
        }
        if options.idempotency_key.is_some() {
            pairs.push("idempotency-key=<set>".to_string());
        }
        for name in self.config.default_headers.keys() {
            pairs.push(format!("{}=<set>", name.as_str().to_ascii_lowercase()));
        }
        for name in options.headers.keys() {
            pairs.push(format!("{}=<set>", name.as_str().to_ascii_lowercase()));
        }
        pairs.sort();
        pairs.dedup();
        pairs.join(", ")
    }
}

fn serialize_json_value<T: Serialize + ?Sized>(value: Option<&T>) -> Result<Option<Value>, Error> {
    value
        .map(serde_json::to_value)
        .transpose()
        .map_err(Error::Serde)
}

fn compact_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"<unserializable-json>\"".to_string())
}

pub struct TypedSseStream<T> {
    inner: SseJsonStream<T>,
}

impl<T> TypedSseStream<T> {
    pub(crate) fn new(inner: SseJsonStream<T>) -> Self {
        Self { inner }
    }

    pub async fn collect_all(mut self) -> Result<Vec<T>, Error>
    where
        T: DeserializeOwned,
    {
        let mut events = Vec::new();
        while let Some(event) = self.inner.next().await {
            events.push(event?);
        }
        Ok(events)
    }
}

impl<T> futures::Stream for TypedSseStream<T>
where
    T: DeserializeOwned,
{
    type Item = Result<T, Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::pin::Pin::new(&mut self.inner).poll_next(cx)
    }
}

async fn parse_json_response<T: DeserializeOwned>(response: Response) -> Result<T, Error> {
    let status = response.status();
    let text = response.text().await?;
    print_response_trace(status.as_u16(), &text);
    if !status.is_success() {
        warn!(
            "openai json response error: status={}, body_len={}",
            status.as_u16(),
            text.len()
        );
        return Err(parse_api_error(status.as_u16(), &text));
    }
    match serde_json::from_str(&text) {
        Ok(parsed) => Ok(parsed),
        Err(err) => {
            warn!(
                "openai json parse failed: status={}, body_len={}, error={}",
                status.as_u16(),
                text.len(),
                err
            );
            Err(Error::Serde(err))
        }
    }
}

fn print_response_trace(status: u16, text: &str) {
    let status_color = if (200..300).contains(&status) {
        ANSI_GREEN
    } else if status >= 400 {
        ANSI_RED
    } else {
        ANSI_YELLOW
    };
    eprintln!(
        "{}[vendor-ai-sdk]{} {}response{} {}status={}{}",
        ANSI_BOLD_CYAN, ANSI_RESET, ANSI_DIM, ANSI_RESET, status_color, status, ANSI_RESET
    );
    eprintln!(
        "{}[vendor-ai-sdk]{} {}response{} {}body={}{}",
        ANSI_BOLD_CYAN,
        ANSI_RESET,
        ANSI_DIM,
        ANSI_RESET,
        ANSI_MAGENTA,
        response_body_preview(text),
        ANSI_RESET
    );
}

fn response_body_preview(text: &str) -> String {
    const MAX_PREVIEW: usize = 800;
    let compact = match serde_json::from_str::<Value>(text) {
        Ok(value) => compact_json(&value),
        Err(_) => text.trim().to_string(),
    };

    if compact.chars().count() <= MAX_PREVIEW {
        return compact;
    }

    let truncated: String = compact.chars().take(MAX_PREVIEW).collect();
    format!("{truncated}...<truncated>")
}

fn should_retry(status: StatusCode) -> bool {
    status == StatusCode::REQUEST_TIMEOUT
        || status == StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error()
}

async fn sleep_retry(attempt: u32) {
    let millis = 100_u64.saturating_mul(2_u64.saturating_pow(attempt.saturating_sub(1)));
    tokio::time::sleep(Duration::from_millis(millis)).await;
}

pub(crate) fn bearer_header_value(
    api_key: &str,
) -> Result<HeaderValue, reqwest::header::InvalidHeaderValue> {
    HeaderValue::from_str(&format!("Bearer {api_key}"))
}

pub(crate) fn path_segment(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('/', "%2F")
        .replace(' ', "%20")
        .replace('#', "%23")
        .replace('?', "%3F")
}
