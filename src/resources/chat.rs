//! Chat Completions 资源封装。
//!
//! 提供 Chat Completions API 的同步和流式调用路径。
//! 对应 OpenAI API 的 `/chat/completions` 端点。
//!
//! ## 使用方式
//!
//! ```no_run
//! use ai_provider_sdk::{OpenAI, ChatCompletionCreateParams, ChatMessage};
//!
//! # async fn example(client: OpenAI) -> ai_provider_sdk::Result<()> {
//! // 同步调用
//! let completion = client
//!     .chat()
//!     .completions()
//!     .create(ChatCompletionCreateParams::new(
//!         "gpt-4.1-mini",
//!         vec![ChatMessage::user("Hello!")],
//!     ))
//!     .await?;
//!
//! // 流式调用
//! let stream = client
//!     .chat()
//!     .completions()
//!     .create_stream(ChatCompletionCreateParams::new(
//!         "gpt-4.1-mini",
//!         vec![ChatMessage::user("Hello!")],
//!     ))
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
use crate::types::{ChatCompletion, ChatCompletionCreateParams};

#[derive(Clone)]
/// Chat 资源入口。
///
/// 通过 `client.chat()` 获取，提供对 Chat API 的命名空间访问。
/// 调用 `completions()` 获取实际请求发送器 [`ChatCompletions`]。
pub struct Chat {
    transport: Arc<Transport>,
}

impl Chat {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    /// 获取 Chat Completions 请求发送器。
    pub fn completions(&self) -> ChatCompletions {
        ChatCompletions {
            transport: self.transport.clone(),
        }
    }
}

/// Chat Completions 请求发送器。
///
/// 提供同步和流式两种调用模式，均支持可选的 [`RequestOptions`] 覆盖。
#[derive(Clone)]
pub struct ChatCompletions {
    transport: Arc<Transport>,
}

impl ChatCompletions {
    /// 创建 Chat Completion（同步模式，等待完整响应）。
    ///
    /// 等价于 `create_with_options(params, RequestOptions::default())`。
    pub async fn create(&self, params: ChatCompletionCreateParams) -> Result<ChatCompletion> {
        self.create_with_options(params, RequestOptions::default())
            .await
    }

    /// 创建 Chat Completion（同步模式，带请求级覆盖项）。
    pub async fn create_with_options(
        &self,
        params: ChatCompletionCreateParams,
        options: RequestOptions,
    ) -> Result<ChatCompletion> {
        self.transport
            .post_json("/chat/completions", request_body(params, false)?, options)
            .await
    }

    /// 创建 Chat Completion 流式请求。
    ///
    /// 返回 [`SseStream`]，调用方通过 `.events()` 消费增量事件流。
    /// 等价于 `create_stream_with_options(params, RequestOptions::default())`。
    pub async fn create_stream(&self, params: ChatCompletionCreateParams) -> Result<SseStream> {
        self.create_stream_with_options(params, RequestOptions::default())
            .await
    }

    /// 创建 Chat Completion 流式请求（带请求级覆盖项）。
    pub async fn create_stream_with_options(
        &self,
        params: ChatCompletionCreateParams,
        options: RequestOptions,
    ) -> Result<SseStream> {
        self.transport
            .post_stream("/chat/completions", request_body(params, true)?, options)
            .await
    }
}

/// 构建请求体 JSON，注入 `stream` 字段。
///
/// `stream` 参数控制服务端返回完整响应（`false`）还是 SSE 事件流（`true`）。
fn request_body(params: ChatCompletionCreateParams, stream: bool) -> Result<Value> {
    let mut body = to_value(params)?;
    if let Value::Object(map) = &mut body {
        map.insert("stream".to_string(), Value::Bool(stream));
    }
    Ok(body)
}
