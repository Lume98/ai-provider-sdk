//! Moderations 资源封装。
//!
//! 处理输入内容的审核请求，支持文本和多模态（文本+图片）输入。
//! 对应 OpenAI API 的 `/moderations` 端点。
//!
//! ## 使用方式
//!
//! ```no_run
//! use ai_provider_sdk::{OpenAI, ModerationCreateParams, ModerationInputItem};
//!
//! # async fn example(client: OpenAI) -> ai_provider_sdk::Result<()> {
//! // 文本审核
//! let result = client
//!     .moderations()
//!     .create(ModerationCreateParams::new("I want to check this text"))
//!     .await?;
//!
//! // 多模态审核（文本 + 图片 URL）
//! let result = client
//!     .moderations()
//!     .create(ModerationCreateParams::new(vec![
//!         ModerationInputItem::text("check this"),
//!         ModerationInputItem::image_url("https://example.com/img.png"),
//!     ]))
//!     .await?;
//!
//! if result.results[0].flagged {
//!     println!("Content flagged!");
//! }
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;

use serde_json::{to_value, Value};

use crate::error::Result;
use crate::request_options::RequestOptions;
use crate::transport::Transport;
use crate::types::{CreateModerationResponse, ModerationCreateParams};

#[derive(Clone)]
/// Moderations 资源入口。
///
/// 通过 `client.moderations()` 获取。
pub struct Moderations {
    transport: Arc<Transport>,
}

impl Moderations {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    /// 创建内容审核请求。
    ///
    /// 等价于 `create_with_options(params, RequestOptions::default())`。
    pub async fn create(&self, params: ModerationCreateParams) -> Result<CreateModerationResponse> {
        self.create_with_options(params, RequestOptions::default())
            .await
    }

    /// 创建内容审核请求（带请求级覆盖项）。
    pub async fn create_with_options(
        &self,
        params: ModerationCreateParams,
        options: RequestOptions,
    ) -> Result<CreateModerationResponse> {
        self.transport
            .post_json("/moderations", request_body(params)?, options)
            .await
    }
}

/// 构建审核请求体 JSON。
fn request_body(params: ModerationCreateParams) -> Result<Value> {
    to_value(params).map_err(Into::into)
}
