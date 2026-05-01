//! 客户端构建与资源访问入口。负责配置归一化、默认头注入与 Transport 装配。

use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};
use url::Url;

use crate::error::{Error, Result};
use crate::resources::{Chat, Embeddings, Files, Models, Moderations, Responses};
use crate::transport::Transport;
use crate::workload::WorkloadIdentity;

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(600);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_MAX_RETRIES: u32 = 2;

#[derive(Debug, Clone)]
/// 客户端初始化选项。
///
/// 边界约束：
/// - `api_key` 可通过显式传入或环境变量 `OPENAI_API_KEY` 提供。
/// - `default_headers` 与 `default_query` 会应用于每次请求。
pub struct ClientOptions {
    pub api_key: Option<String>,
    pub workload_identity: Option<WorkloadIdentity>,
    pub organization: Option<String>,
    pub project: Option<String>,
    pub webhook_secret: Option<String>,
    pub base_url: Option<String>,
    pub websocket_base_url: Option<String>,
    pub timeout: Option<Duration>,
    pub max_retries: u32,
    pub default_headers: Option<HashMap<String, String>>,
    pub default_query: Option<HashMap<String, String>>,
    pub _strict_response_validation: bool,
    // TODO: 支持外部传入自定义 reqwest::Client，用于自定义代理、TLS、连接池等
    // pub http_client: Option<reqwest::Client>,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            api_key: None,
            workload_identity: None,
            organization: None,
            project: None,
            webhook_secret: None,
            base_url: None,
            websocket_base_url: None,
            timeout: Some(DEFAULT_TIMEOUT),
            max_retries: DEFAULT_MAX_RETRIES,
            default_headers: None,
            default_query: None,
            _strict_response_validation: false,
        }
    }
}

#[derive(Clone)]
pub struct OpenAI {
    pub(crate) inner: Arc<Transport>,
}

impl OpenAI {
    /// 使用显式 API Key 创建客户端。
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_options(ClientOptions {
            api_key: Some(api_key.into()),
            ..ClientOptions::default()
        })
    }

    /// 仅从环境变量读取配置创建客户端。
    pub fn from_env() -> Result<Self> {
        Self::with_options(ClientOptions::default())
    }

    /// 使用完整选项创建客户端并完成配置归一化。
    pub fn with_options(mut options: ClientOptions) -> Result<Self> {
        if options.api_key.is_some() && options.workload_identity.is_some() {
            return Err(Error::Config(
                "The `api_key` and `workload_identity` arguments are mutually exclusive"
                    .to_string(),
            ));
        }

        let api_key = if options.workload_identity.is_some() {
            String::new()
        } else {
            options
                .api_key
                .take()
                .or_else(|| env::var("OPENAI_API_KEY").ok())
                .ok_or_else(|| {
                    Error::Config(
                        "api_key must be provided or OPENAI_API_KEY must be set".to_string(),
                    )
                })?
        };

        if options.organization.is_none() {
            options.organization = env::var("OPENAI_ORG_ID").ok();
        }
        if options.project.is_none() {
            options.project = env::var("OPENAI_PROJECT_ID").ok();
        }
        if options.webhook_secret.is_none() {
            options.webhook_secret = env::var("OPENAI_WEBHOOK_SECRET").ok();
        }

        let base_url = options
            .base_url
            .take()
            .or_else(|| env::var("OPENAI_BASE_URL").ok())
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        let base_url = normalize_base_url(&base_url)?;
        let headers = build_default_headers(&api_key, &options)?;
        let mut http_builder = reqwest::Client::builder()
            .connect_timeout(DEFAULT_CONNECT_TIMEOUT);
        if let Some(timeout) = options.timeout {
            http_builder = http_builder.timeout(timeout);
        }
        let http = http_builder
            .build()
            .map_err(|err| Error::Connection(err.to_string()))?;

        Ok(Self {
            inner: Arc::new(Transport::new(
                http,
                base_url,
                headers,
                options.default_query.unwrap_or_default(),
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

    if let Some(default_headers) = &options.default_headers {
        for (key, value) in default_headers {
            let name = HeaderName::from_bytes(key.as_bytes())
                .map_err(|err| Error::Config(format!("invalid header name `{key}`: {err}")))?;
            headers.insert(name, HeaderValue::from_str(value)?);
        }
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
