use std::{env, time::Duration};

use bytes::Bytes;
use futures::StreamExt;
use reqwest::{
    Client, Method, Response, StatusCode,
    header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue},
    multipart,
};
use serde::{Serialize, de::DeserializeOwned};

use crate::{Error, parse_api_error, sse::SseJsonStream};

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const USER_AGENT: &str = concat!("vendor-ai-sdk/", env!("CARGO_PKG_VERSION"));

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
        let mut attempt = 0;
        loop {
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
                    sleep_retry(attempt).await;
                }
                Ok(response) => return Ok(response),
                Err(err)
                    if (err.is_timeout() || err.is_connect())
                        && attempt < self.config.max_retries =>
                {
                    attempt += 1;
                    sleep_retry(attempt).await;
                }
                Err(err) => return Err(Error::Http(err)),
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
        let response = self
            .build(Method::POST, path, Option::<&()>::None, options)?
            .multipart(form)
            .send()
            .await?;
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
        let response = self.json(method, path, query, body, options).await?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await?;
            return Err(parse_api_error(status.as_u16(), &text));
        }
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
    if !status.is_success() {
        return Err(parse_api_error(status.as_u16(), &text));
    }
    Ok(serde_json::from_str(&text)?)
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
