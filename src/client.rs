use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};
use url::Url;

use crate::error::{Error, Result};
use crate::resources::{Chat, Embeddings, Files, Models, Moderations, Responses};
use crate::transport::Transport;

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(600);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_MAX_RETRIES: u32 = 2;

#[derive(Debug, Clone)]
pub struct ClientOptions {
    pub api_key: Option<String>,
    pub organization: Option<String>,
    pub project: Option<String>,
    pub base_url: Option<String>,
    pub timeout: Duration,
    pub max_retries: u32,
    pub default_headers: HashMap<String, String>,
    pub default_query: HashMap<String, String>,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            api_key: None,
            organization: None,
            project: None,
            base_url: None,
            timeout: DEFAULT_TIMEOUT,
            max_retries: DEFAULT_MAX_RETRIES,
            default_headers: HashMap::new(),
            default_query: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct OpenAI {
    pub(crate) inner: Arc<Transport>,
}

impl OpenAI {
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_options(ClientOptions {
            api_key: Some(api_key.into()),
            ..ClientOptions::default()
        })
    }

    pub fn from_env() -> Result<Self> {
        Self::with_options(ClientOptions::default())
    }

    pub fn with_options(mut options: ClientOptions) -> Result<Self> {
        let api_key = options
            .api_key
            .take()
            .or_else(|| env::var("OPENAI_API_KEY").ok())
            .ok_or_else(|| {
                Error::Config("api_key must be provided or OPENAI_API_KEY must be set".to_string())
            })?;

        if options.organization.is_none() {
            options.organization = env::var("OPENAI_ORG_ID").ok();
        }
        if options.project.is_none() {
            options.project = env::var("OPENAI_PROJECT_ID").ok();
        }

        let base_url = options
            .base_url
            .take()
            .or_else(|| env::var("OPENAI_BASE_URL").ok())
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        let base_url = normalize_base_url(&base_url)?;
        let headers = build_default_headers(&api_key, &options)?;
        let http = reqwest::Client::builder()
            .timeout(options.timeout)
            .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
            .build()
            .map_err(|err| Error::Connection(err.to_string()))?;

        Ok(Self {
            inner: Arc::new(Transport::new(
                http,
                base_url,
                headers,
                options.default_query,
                options.max_retries,
            )),
        })
    }

    pub fn responses(&self) -> Responses {
        Responses::new(self.inner.clone())
    }

    pub fn chat(&self) -> Chat {
        Chat::new(self.inner.clone())
    }

    pub fn models(&self) -> Models {
        Models::new(self.inner.clone())
    }

    pub fn embeddings(&self) -> Embeddings {
        Embeddings::new(self.inner.clone())
    }

    pub fn files(&self) -> Files {
        Files::new(self.inner.clone())
    }

    pub fn moderations(&self) -> Moderations {
        Moderations::new(self.inner.clone())
    }
}

fn normalize_base_url(base_url: &str) -> Result<Url> {
    let mut url = Url::parse(base_url)?;
    if !url.path().ends_with('/') {
        let path = format!("{}/", url.path().trim_end_matches('/'));
        url.set_path(&path);
    }
    Ok(url)
}

fn build_default_headers(api_key: &str, options: &ClientOptions) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {api_key}"))?,
    );
    headers.insert("x-stainless-async", HeaderValue::from_static("true"));
    headers.insert("content-type", HeaderValue::from_static("application/json"));

    if let Some(organization) = &options.organization {
        headers.insert("openai-organization", HeaderValue::from_str(organization)?);
    }
    if let Some(project) = &options.project {
        headers.insert("openai-project", HeaderValue::from_str(project)?);
    }

    for (key, value) in &options.default_headers {
        let name = HeaderName::from_bytes(key.as_bytes())
            .map_err(|err| Error::Config(format!("invalid header name `{key}`: {err}")))?;
        headers.insert(name, HeaderValue::from_str(value)?);
    }

    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_base_url_with_trailing_slash() {
        let url = normalize_base_url("https://api.example.com/v1").unwrap();
        assert_eq!(url.as_str(), "https://api.example.com/v1/");
    }
}
