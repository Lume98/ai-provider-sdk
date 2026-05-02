//! 客户端构建与资源访问入口。
//!
//! 本模块负责：
//! - 归一化配置（API Key 优先级、环境变量回退、base URL 处理）
//! - 构建全局默认 HTTP 头（鉴权、组织/项目标识）
//! - 组装 [`Transport`] 并通过 `Arc` 共享给各资源模块
//!
//! ## 示例
//!
//! ```no_run
//! use ai_provider_sdk::{OpenAI, ClientOptions};
//!
//! // 方式一：显式 API Key
//! let client = OpenAI::new("sk-...")?;
//!
//! // 方式二：从环境变量读取
//! let client = OpenAI::from_env()?;
//!
//! // 方式三：完整选项
//! let client = OpenAI::with_options(ClientOptions {
//!     api_key: Some("sk-...".into()),
//!     organization: Some("org_123".into()),
//!     ..ClientOptions::default()
//! })?;
//! # Ok::<(), ai_provider_sdk::Error>(())
//! ```

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

/// OpenAI API 默认基础 URL。
const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

/// 默认总请求超时（含 DNS 解析、连接、首字节、body 读取）。
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(600);

/// 默认 TCP 连接超时。
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// 默认最大重试次数（仅对可重试状态码和连接错误生效）。
const DEFAULT_MAX_RETRIES: u32 = 2;

#[derive(Debug, Clone)]
/// 客户端初始化选项。
///
/// 边界约束：
/// - `api_key` 与 `workload_identity` 互斥，同时传入将返回配置错误。
/// - `api_key` 可通过显式传入或环境变量 `OPENAI_API_KEY` 提供；优先使用显式值。
/// - `default_headers` 与 `default_query` 会应用于每次请求（可被单次 `RequestOptions` 覆盖）。
///
/// ## 环境变量映射
///
/// | 字段               | 环境变量                |
/// |-------------------|-----------------------|
/// | `api_key`         | `OPENAI_API_KEY`      |
/// | `organization`    | `OPENAI_ORG_ID`       |
/// | `project`         | `OPENAI_PROJECT_ID`   |
/// | `webhook_secret`  | `OPENAI_WEBHOOK_SECRET`|
/// | `base_url`        | `OPENAI_BASE_URL`     |
pub struct ClientOptions {
    /// API 密钥。与 `workload_identity` 互斥。
    pub api_key: Option<String>,
    /// Workload Identity 联合身份认证配置。与 `api_key` 互斥。
    pub workload_identity: Option<WorkloadIdentity>,
    /// OpenAI 组织 ID，会以 `openai-organization` 头发送。
    pub organization: Option<String>,
    /// OpenAI 项目 ID，会以 `openai-project` 头发送。
    pub project: Option<String>,
    /// Webhook 签名密钥（当前预留，未在请求链路中使用）。
    pub webhook_secret: Option<String>,
    /// 自定义 base URL，用于对接兼容 OpenAI API 的第三方服务。
    pub base_url: Option<String>,
    /// WebSocket 基础 URL（当前预留）。
    pub websocket_base_url: Option<String>,
    /// 请求总超时。为 `None` 时使用默认值 600 秒。
    pub timeout: Option<Duration>,
    /// 最大重试次数。0 表示不重试。
    pub max_retries: u32,
    /// 追加到每次请求的默认 HTTP 头。
    pub default_headers: Option<HashMap<String, String>>,
    /// 追加到每次请求的默认查询参数。
    pub default_query: Option<HashMap<String, String>>,
    /// 是否严格校验响应体结构（当前预留）。
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
/// SDK 主入口客户端。
///
/// 内部持有 `Arc<Transport>`，克隆开销极小，可安全跨任务共享。
/// 通过 `chat()`、`models()`、`embeddings()` 等方法获取对应资源入口。
pub struct OpenAI {
    pub(crate) inner: Arc<Transport>,
}

impl OpenAI {
    /// 使用显式 API Key 创建客户端。
    ///
    /// 等价于 `OpenAI::with_options(ClientOptions { api_key: Some(key), .. })`。
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_options(ClientOptions {
            api_key: Some(api_key.into()),
            ..ClientOptions::default()
        })
    }

    /// 仅从环境变量读取配置创建客户端。
    ///
    /// 需要设置 `OPENAI_API_KEY` 环境变量，否则返回配置错误。
    pub fn from_env() -> Result<Self> {
        Self::with_options(ClientOptions::default())
    }

    /// 使用完整选项创建客户端并完成配置归一化。
    ///
    /// 归一化流程：
    /// 1. 校验 `api_key` 与 `workload_identity` 互斥。
    /// 2. 从选项或环境变量解析 API Key。
    /// 3. 从环境变量回填 `organization` / `project` / `webhook_secret`。
    /// 4. 归一化 `base_url`（确保以 `/` 结尾）。
    /// 5. 构建默认 HTTP 头（含鉴权头、 Stainless 异步标记）。
    /// 6. 构建 `reqwest::Client` 并组装 `Transport`。
    pub fn with_options(mut options: ClientOptions) -> Result<Self> {
        // api_key 和 workload_identity 不能同时指定
        if options.api_key.is_some() && options.workload_identity.is_some() {
            return Err(Error::Config(
                "The `api_key` and `workload_identity` arguments are mutually exclusive"
                    .to_string(),
            ));
        }

        // 确定认证方式：workload_identity 模式不需要静态 key
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

        // 从环境变量回填可选配置（仅当选项中未显式指定时）
        if options.organization.is_none() {
            options.organization = env::var("OPENAI_ORG_ID").ok();
        }
        if options.project.is_none() {
            options.project = env::var("OPENAI_PROJECT_ID").ok();
        }
        if options.webhook_secret.is_none() {
            options.webhook_secret = env::var("OPENAI_WEBHOOK_SECRET").ok();
        }

        // 解析并归一化 base_url
        let base_url = options
            .base_url
            .take()
            .or_else(|| env::var("OPENAI_BASE_URL").ok())
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        let base_url = normalize_base_url(&base_url)?;

        // 构建全局默认 HTTP 头
        let headers = build_default_headers(&api_key, &options)?;

        // 构建 reqwest::Client，设置连接超时和请求超时
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

    /// Responses API 资源入口。
    ///
    /// 用于创建和管理 OpenAI Responses（新一代对话 API）。
    pub fn responses(&self) -> Responses {
        Responses::new(self.inner.clone())
    }

    /// Chat Completions API 资源入口。
    ///
    /// 通过 `client.chat().completions()` 访问聊天补全接口。
    pub fn chat(&self) -> Chat {
        Chat::new(self.inner.clone())
    }

    /// Models API 资源入口。
    ///
    /// 用于列出可用模型和查询模型详情。
    pub fn models(&self) -> Models {
        Models::new(self.inner.clone())
    }

    /// Embeddings API 资源入口。
    ///
    /// 用于将文本转换为向量表示。
    pub fn embeddings(&self) -> Embeddings {
        Embeddings::new(self.inner.clone())
    }

    /// Files API 资源入口。
    ///
    /// 用于上传、查询、下载和删除文件（用于 fine-tuning、batch 等场景）。
    pub fn files(&self) -> Files {
        Files::new(self.inner.clone())
    }

    /// Moderations API 资源入口。
    ///
    /// 用于审核文本和图片内容是否违反使用政策。
    pub fn moderations(&self) -> Moderations {
        Moderations::new(self.inner.clone())
    }
}

/// 归一化 base URL，确保路径以 `/` 结尾。
///
/// 这是因为 `url::Url::join` 的行为依赖于基路径是否以 `/` 结尾：
/// - `https://api.com/v1/` + `chat/completions` → `https://api.com/v1/chat/completions` ✓
/// - `https://api.com/v1` + `chat/completions` → `https://api.com/chat/completions` ✗
fn normalize_base_url(base_url: &str) -> Result<Url> {
    let mut url = Url::parse(base_url)?;
    if !url.path().ends_with('/') {
        let path = format!("{}/", url.path().trim_end_matches('/'));
        url.set_path(&path);
    }
    Ok(url)
}

/// 构建全局默认 HTTP 头集合。
///
/// 固定头：
/// - `Authorization: Bearer {api_key}`
/// - `x-stainless-async: true`（标识 SDK 为异步实现）
/// - `Content-Type: application/json`
///
/// 条件头（当选项中指定时追加）：
/// - `openai-organization`
/// - `openai-project`
///
/// 用户自定义头会覆盖同名固定头。
fn build_default_headers(api_key: &str, options: &ClientOptions) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {api_key}"))?,
    );
    headers.insert("x-stainless-async", HeaderValue::from_static("true"));
    headers.insert("content-type", HeaderValue::from_static("application/json"));

    // 条件追加组织和项目标识头
    if let Some(organization) = &options.organization {
        headers.insert("openai-organization", HeaderValue::from_str(organization)?);
    }
    if let Some(project) = &options.project {
        headers.insert("openai-project", HeaderValue::from_str(project)?);
    }

    // 用户自定义头可覆盖上方默认值
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
