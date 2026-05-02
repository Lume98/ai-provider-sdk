//! Models 资源封装。
//!
//! 提供模型列表与模型详情查询。
//! 对应 OpenAI API 的 `/models` 和 `/models/{model}` 端点。
//!
//! ## 使用方式
//!
//! ```no_run
//! use ai_provider_sdk::OpenAI;
//!
//! # async fn example(client: OpenAI) -> ai_provider_sdk::Result<()> {
//! // 列出所有可用模型
//! let models = client.models().list().await?;
//! for model in &models.data {
//!     println!("{} (owned by {:?})", model.id, model.owned_by);
//! }
//!
//! // 查询单个模型详情
//! let model = client.models().retrieve("gpt-4.1-mini").await?;
//! println!("{:?}", model);
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;

use crate::error::{Error, Result};
use crate::path::encode_path_segment;
use crate::request_options::RequestOptions;
use crate::transport::Transport;
use crate::types::{Model, ModelList};

#[derive(Clone)]
/// Models 资源入口。
///
/// 通过 `client.models()` 获取。
pub struct Models {
    transport: Arc<Transport>,
}

impl Models {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    /// 列出所有可用模型。
    ///
    /// 等价于 `list_with_options(RequestOptions::default())`。
    pub async fn list(&self) -> Result<ModelList> {
        self.list_with_options(RequestOptions::default()).await
    }

    /// 列出所有可用模型（带请求级覆盖项）。
    pub async fn list_with_options(&self, options: RequestOptions) -> Result<ModelList> {
        self.transport.get_json("/models", options).await
    }

    /// 查询指定模型的详情。
    ///
    /// 模型 ID 会做 URL 安全编码，支持包含 `/` 和空格的名称
    /// （如 `"fine/tuned model"` → `/models/fine%2Ftuned%20model`）。
    pub async fn retrieve(&self, model: impl AsRef<str>) -> Result<Model> {
        self.retrieve_with_options(model, RequestOptions::default())
            .await
    }

    /// 查询指定模型的详情（带请求级覆盖项）。
    ///
    /// 空模型 ID 会返回配置错误，避免发出无效请求。
    pub async fn retrieve_with_options(
        &self,
        model: impl AsRef<str>,
        options: RequestOptions,
    ) -> Result<Model> {
        let model = model.as_ref();
        if model.is_empty() {
            return Err(Error::Config("model must not be empty".to_string()));
        }

        let model = encode_path_segment(model);
        self.transport
            .get_json(&format!("/models/{model}"), options)
            .await
    }
}
