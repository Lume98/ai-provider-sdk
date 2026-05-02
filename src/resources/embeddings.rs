//! Embeddings 资源封装。
//!
//! 处理文本向量生成请求，自动填充默认编码格式。
//! 对应 OpenAI API 的 `/embeddings` 端点。
//!
//! ## 使用方式
//!
//! ```no_run
//! use ai_provider_sdk::{OpenAI, EmbeddingCreateParams};
//!
//! # async fn example(client: OpenAI) -> ai_provider_sdk::Result<()> {
//! let response = client
//!     .embeddings()
//!     .create(EmbeddingCreateParams::new("text-embedding-3-small", "hello"))
//!     .await?;
//!
//! for embedding in &response.data {
//!     println!("index={}", embedding.index);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## 默认行为
//!
//! 当 `encoding_format` 未指定时，自动填充为 `EncodingFormat::Float`，
//! 与 OpenAI API 的默认行为一致。

use std::sync::Arc;

use serde_json::{to_value, Value};

use crate::error::Result;
use crate::request_options::RequestOptions;
use crate::transport::Transport;
use crate::types::{CreateEmbeddingResponse, EmbeddingCreateParams, EncodingFormat};

#[derive(Clone)]
/// Embeddings 资源入口。
///
/// 通过 `client.embeddings()` 获取。
pub struct Embeddings {
    transport: Arc<Transport>,
}

impl Embeddings {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    /// 创建 Embedding 请求。
    ///
    /// 等价于 `create_with_options(params, RequestOptions::default())`。
    pub async fn create(&self, params: EmbeddingCreateParams) -> Result<CreateEmbeddingResponse> {
        self.create_with_options(params, RequestOptions::default())
            .await
    }

    /// 创建 Embedding 请求（带请求级覆盖项）。
    pub async fn create_with_options(
        &self,
        params: EmbeddingCreateParams,
        options: RequestOptions,
    ) -> Result<CreateEmbeddingResponse> {
        self.transport
            .post_json("/embeddings", request_body(params)?, options)
            .await
    }
}

/// 构建请求体 JSON，自动填充默认编码格式。
///
/// 当 `encoding_format` 为 `None` 时，设置为 `Float`，
/// 确保服务端返回可读的浮点数组而非 base64 字符串。
fn request_body(mut params: EmbeddingCreateParams) -> Result<Value> {
    if params.encoding_format.is_none() {
        params.encoding_format = Some(EncodingFormat::Float);
    }

    to_value(params).map_err(Into::into)
}
