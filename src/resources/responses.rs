//! Responses 资源封装。
//!
//! 统一普通响应与流式响应调用路径。
//! 对应 OpenAI API 的 `/responses` 端点（新一代对话 API）。
//!
//! ## 使用方式
//!
//! ```no_run
//! use ai_provider_sdk::{OpenAI, ResponseCreateParams};
//!
//! # async fn example(client: OpenAI) -> ai_provider_sdk::Result<()> {
//! // 同步调用
//! let response = client
//!     .responses()
//!     .create(ResponseCreateParams::new("gpt-4.1-mini").input("Hello!"))
//!     .await?;
//!
//! // 流式调用
//! let stream = client
//!     .responses()
//!     .create_stream(ResponseCreateParams::new("gpt-4.1-mini").input("Hello!"))
//!     .await?;
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;

use serde_json::{to_value, Value};

use crate::error::Result;
use crate::request_options::RequestOptions;
use crate::streaming::SseStream;
use crate::transport::Transport;
use crate::types::{Response, ResponseCreateParams};

#[derive(Clone)]
/// Responses 资源入口。
///
/// 通过 `client.responses()` 获取。
pub struct Responses {
    transport: Arc<Transport>,
}

impl Responses {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    /// 创建 Response（同步模式）。
    ///
    /// 等价于 `create_with_options(params, RequestOptions::default())`。
    pub async fn create(&self, params: ResponseCreateParams) -> Result<Response> {
        self.create_with_options(params, RequestOptions::default())
            .await
    }

    /// 创建 Response（同步模式，带请求级覆盖项）。
    pub async fn create_with_options(
        &self,
        params: ResponseCreateParams,
        options: RequestOptions,
    ) -> Result<Response> {
        self.transport
            .post_json("/responses", request_body(params, false)?, options)
            .await
    }

    /// 创建 Response 流式请求。
    ///
    /// 返回 [`SseStream`]，调用方通过 `.events()` 消费增量事件流。
    pub async fn create_stream(&self, params: ResponseCreateParams) -> Result<SseStream> {
        self.create_stream_with_options(params, RequestOptions::default())
            .await
    }

    /// 创建 Response 流式请求（带请求级覆盖项）。
    pub async fn create_stream_with_options(
        &self,
        params: ResponseCreateParams,
        options: RequestOptions,
    ) -> Result<SseStream> {
        self.transport
            .post_stream("/responses", request_body(params, true)?, options)
            .await
    }
}

/// 构建请求体 JSON，注入 `stream` 字段。
///
/// `stream` 参数控制服务端返回完整响应（`false`）还是 SSE 事件流（`true`）。
fn request_body(params: ResponseCreateParams, stream: bool) -> Result<Value> {
    let mut body = to_value(params)?;
    if let Value::Object(map) = &mut body {
        map.insert("stream".to_string(), Value::Bool(stream));
    }
    Ok(body)
}
