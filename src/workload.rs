//! Workload Identity 联合身份认证类型。
//!
//! 允许 SDK 在不支持静态 API Key 的环境中（如 GKE、EKS、Cloud Run）
//! 通过云厂商的 Workload Identity 机制自动获取访问令牌。
//!
//! 参见 Python SDK `openai/auth/_workload.py`。
//!
//! ## 使用方式
//!
//! ```no_run
//! use ai_provider_sdk::{
//!     ClientOptions, OpenAI, WorkloadIdentity,
//!     SubjectTokenProvider, SubjectTokenType,
//! };
//!
//! let client = OpenAI::with_options(ClientOptions {
//!     workload_identity: Some(WorkloadIdentity {
//!         client_id: "my-client-id".into(),
//!         identity_provider_id: "projects/123/locations/global/workloadIdentityPools/...".into(),
//!         service_account_id: "projects/123/serviceAccounts/my-sa@...".into(),
//!         provider: SubjectTokenProvider::file(
//!             "/var/run/secrets/kubernetes.io/serviceaccount/token",
//!             SubjectTokenType::Jwt,
//!         ),
//!         refresh_buffer_seconds: Some(1200.0),
//!     }),
//!     ..ClientOptions::default()
//! })?;
//! # Ok::<(), ai_provider_sdk::Error>(())
//! ```

#![allow(dead_code)]

use std::fmt;
use std::sync::Arc;

use crate::error::Result;

/// Subject token 类型。
///
/// 不同的身份提供商返回的 token 格式不同：
/// - `Jwt`：Kubernetes service account token（OIDC JWT）。
/// - `Id`：SAML/OIDC ID token。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubjectTokenType {
    /// JWT 格式的 subject token。
    Jwt,
    /// ID token 格式的 subject token。
    Id,
}

/// Subject token 获取器。
///
/// 封装了 token 的获取方式，支持两种模式：
/// - [`SubjectTokenProvider::file`] — 从文件系统读取（如 K8s service account token）。
/// - [`SubjectTokenProvider::from_fn`] — 从自定义闭包获取（适用于动态 token 场景）。
///
/// 内部使用 `Arc<dyn Fn>` 实现，可安全克隆和跨任务共享。
pub struct SubjectTokenProvider {
    /// token 类型，影响后续 token 交换请求的参数。
    pub token_type: SubjectTokenType,
    /// token 获取闭包；每次调用返回当前有效的 subject token。
    get_token: Arc<dyn Fn() -> Result<String> + Send + Sync>,
}

impl SubjectTokenProvider {
    /// 从文件路径读取 subject token。
    ///
    /// 每次调用 `get_token()` 都会重新读取文件内容，适用于 K8s 中
    /// 自动轮转的 service account token（如 `/var/run/secrets/kubernetes.io/serviceaccount/token`）。
    ///
    /// 文件内容会自动 `trim()`，移除尾部换行符。
    pub fn file(path: impl Into<String>, token_type: SubjectTokenType) -> Self {
        let path = path.into();
        Self {
            token_type,
            get_token: Arc::new(move || {
                std::fs::read_to_string(&path)
                    .map(|s| s.trim().to_string())
                    .map_err(|e| {
                        crate::error::Error::Config(format!(
                            "failed to read subject token from {path}: {e}"
                        ))
                    })
            }),
        }
    }

    /// 从自定义函数获取 subject token。
    ///
    /// 适用于 token 来源为环境变量、外部进程输出或自定义 HTTP 调用的场景。
    pub fn from_fn(
        token_type: SubjectTokenType,
        f: impl Fn() -> Result<String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            token_type,
            get_token: Arc::new(f),
        }
    }

    /// 获取当前的 subject token。
    #[allow(dead_code)]
    pub(crate) fn get_token(&self) -> Result<String> {
        (self.get_token)()
    }
}

impl fmt::Debug for SubjectTokenProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubjectTokenProvider")
            .field("token_type", &self.token_type)
            .finish_non_exhaustive()
    }
}

impl Clone for SubjectTokenProvider {
    fn clone(&self) -> Self {
        Self {
            token_type: self.token_type.clone(),
            get_token: Arc::clone(&self.get_token),
        }
    }
}

/// Workload Identity 联合身份配置。
///
/// 与 `api_key` 互斥——通过 [`ClientOptions`](crate::ClientOptions) 构建客户端时
/// 只能指定二者之一。
///
/// ## 认证流程
///
/// 1. 通过 `provider` 获取 subject token。
/// 2. 使用 `client_id`、`identity_provider_id`、`service_account_id` 构造 token 交换请求。
/// 3. 向 Google STS API 交换获取短期访问令牌。
/// 4. 使用访问令牌作为 API 请求的 `Bearer` 凭证。
#[derive(Debug, Clone)]
pub struct WorkloadIdentity {
    /// OAuth 客户端 ID，标识发起请求的应用。
    pub client_id: String,
    /// WIFAPI 中的身份提供商资源 ID（如 GCP Workload Identity Pool Provider）。
    pub identity_provider_id: String,
    /// 绑定已验证外部身份的服务账号 ID（如 GCP Service Account）。
    pub service_account_id: String,
    /// Subject token 获取器，负责从运行环境读取或计算 token。
    pub provider: SubjectTokenProvider,
    /// token 过期前提前刷新的缓冲时间（秒），默认 1200（20 分钟）。
    ///
    /// 避免在 token 即将过期时发起请求，导致请求途中 token 失效。
    pub refresh_buffer_seconds: Option<f64>,
}
