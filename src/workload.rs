//! Workload Identity 联合身份认证类型。
//!
//! 参见 Python SDK `openai/auth/_workload.py`。
#![allow(dead_code)]

use std::fmt;
use std::sync::Arc;

use crate::error::Result;

/// Subject token 类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubjectTokenType {
    Jwt,
    Id,
}

/// Subject token 获取器。
///
/// 可通过辅助函数构造：
/// - `SubjectTokenProvider::file(path)` — 从文件读取 token（如 K8s service account token）
/// - `SubjectTokenProvider::from_fn(f)` — 从自定义闭包获取 token
pub struct SubjectTokenProvider {
    pub token_type: SubjectTokenType,
    get_token: Arc<dyn Fn() -> Result<String> + Send + Sync>,
}

impl SubjectTokenProvider {
    /// 从文件读取 subject token（如 K8s `/var/run/secrets/kubernetes.io/serviceaccount/token`）。
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
    pub fn from_fn(
        token_type: SubjectTokenType,
        f: impl Fn() -> Result<String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            token_type,
            get_token: Arc::new(f),
        }
    }

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
/// 与 `api_key` 互斥——二者只能传其一。
#[derive(Debug, Clone)]
pub struct WorkloadIdentity {
    /// 客户端 ID。
    pub client_id: String,
    /// WIFAPI 中的身份提供商资源 ID。
    pub identity_provider_id: String,
    /// 绑定已验证外部身份的服务账号 ID。
    pub service_account_id: String,
    /// Subject token 获取器。
    pub provider: SubjectTokenProvider,
    /// token 过期前提前刷新的缓冲时间（秒），默认 1200（20 分钟）。
    pub refresh_buffer_seconds: Option<f64>,
}
