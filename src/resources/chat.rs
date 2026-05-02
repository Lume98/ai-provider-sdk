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

use serde_json::{Value, to_value};

use crate::error::{Error, Result};
use crate::pagination::CursorPage;
use crate::path::encode_path_segment;
use crate::request_options::RequestOptions;
use crate::streaming::SseStream;
use crate::transport::Transport;
use crate::types::{
    ChatCompletion, ChatCompletionCreateParams, ChatCompletionDeleted, ChatCompletionListParams,
    ChatCompletionMessageListParams, ChatCompletionStoreMessage, ChatCompletionUpdateParams,
    ChatListOrder,
};

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

    /// 获取 stored Chat Completion。
    ///
    /// 仅 `store=true` 创建的 completion 可被服务端返回。
    pub async fn retrieve(&self, completion_id: impl AsRef<str>) -> Result<ChatCompletion> {
        self.retrieve_with_options(completion_id, RequestOptions::default())
            .await
    }

    /// 获取 stored Chat Completion（带请求级覆盖项）。
    pub async fn retrieve_with_options(
        &self,
        completion_id: impl AsRef<str>,
        options: RequestOptions,
    ) -> Result<ChatCompletion> {
        let path = completion_path(completion_id.as_ref(), None)?;
        self.transport.get_json(&path, options).await
    }

    /// 更新 stored Chat Completion 的 metadata。
    pub async fn update(
        &self,
        completion_id: impl AsRef<str>,
        params: ChatCompletionUpdateParams,
    ) -> Result<ChatCompletion> {
        self.update_with_options(completion_id, params, RequestOptions::default())
            .await
    }

    /// 更新 stored Chat Completion 的 metadata（带请求级覆盖项）。
    pub async fn update_with_options(
        &self,
        completion_id: impl AsRef<str>,
        params: ChatCompletionUpdateParams,
        options: RequestOptions,
    ) -> Result<ChatCompletion> {
        let path = completion_path(completion_id.as_ref(), None)?;
        self.transport
            .post_json(&path, to_value(params)?, options)
            .await
    }

    /// 拉取 stored Chat Completions 列表（默认参数）。
    pub async fn list(&self) -> Result<CursorPage<ChatCompletion>> {
        self.list_with_params(ChatCompletionListParams::default())
            .await
    }

    /// 按显式分页参数拉取 stored Chat Completions 列表。
    pub async fn list_with_params(
        &self,
        params: ChatCompletionListParams,
    ) -> Result<CursorPage<ChatCompletion>> {
        self.list_with_options(params, RequestOptions::default())
            .await
    }

    /// 按参数与请求级覆盖项拉取 stored Chat Completions 列表。
    pub async fn list_with_options(
        &self,
        params: ChatCompletionListParams,
        options: RequestOptions,
    ) -> Result<CursorPage<ChatCompletion>> {
        let options = apply_completion_list_params(params, options);
        self.transport.get_json("/chat/completions", options).await
    }

    /// 删除 stored Chat Completion。
    pub async fn delete(&self, completion_id: impl AsRef<str>) -> Result<ChatCompletionDeleted> {
        self.delete_with_options(completion_id, RequestOptions::default())
            .await
    }

    /// 删除 stored Chat Completion（带请求级覆盖项）。
    pub async fn delete_with_options(
        &self,
        completion_id: impl AsRef<str>,
        options: RequestOptions,
    ) -> Result<ChatCompletionDeleted> {
        let path = completion_path(completion_id.as_ref(), None)?;
        self.transport.delete_json(&path, options).await
    }

    /// 获取 stored Chat Completion 的消息子资源。
    pub fn messages(&self) -> ChatCompletionMessages {
        ChatCompletionMessages {
            transport: self.transport.clone(),
        }
    }
}

/// Stored Chat Completion 消息请求发送器。
#[derive(Clone)]
pub struct ChatCompletionMessages {
    transport: Arc<Transport>,
}

impl ChatCompletionMessages {
    /// 拉取 stored Chat Completion 的消息列表（默认参数）。
    pub async fn list(
        &self,
        completion_id: impl AsRef<str>,
    ) -> Result<CursorPage<ChatCompletionStoreMessage>> {
        self.list_with_params(completion_id, ChatCompletionMessageListParams::default())
            .await
    }

    /// 按显式分页参数拉取 stored Chat Completion 的消息列表。
    pub async fn list_with_params(
        &self,
        completion_id: impl AsRef<str>,
        params: ChatCompletionMessageListParams,
    ) -> Result<CursorPage<ChatCompletionStoreMessage>> {
        self.list_with_options(completion_id, params, RequestOptions::default())
            .await
    }

    /// 按参数与请求级覆盖项拉取 stored Chat Completion 的消息列表。
    pub async fn list_with_options(
        &self,
        completion_id: impl AsRef<str>,
        params: ChatCompletionMessageListParams,
        options: RequestOptions,
    ) -> Result<CursorPage<ChatCompletionStoreMessage>> {
        let path = completion_path(completion_id.as_ref(), Some("messages"))?;
        let options = apply_message_list_params(params, options);
        self.transport.get_json(&path, options).await
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

/// 把 ChatCompletionListParams 写入查询参数，且不覆盖调用方已存在的同名参数。
fn apply_completion_list_params(
    params: ChatCompletionListParams,
    mut options: RequestOptions,
) -> RequestOptions {
    insert_query_if_absent(&mut options, "after", params.after);
    insert_query_if_absent(
        &mut options,
        "limit",
        params.limit.map(|value| value.to_string()),
    );
    insert_query_if_absent(&mut options, "model", params.model);
    insert_query_if_absent(
        &mut options,
        "order",
        params.order.map(order_as_query_value),
    );

    if let Some(metadata) = params.metadata {
        for (key, value) in metadata {
            insert_query_if_absent(&mut options, &format!("metadata[{key}]"), Some(value));
        }
    }

    options
}

/// 把 ChatCompletionMessageListParams 写入查询参数，且不覆盖调用方已存在的同名参数。
fn apply_message_list_params(
    params: ChatCompletionMessageListParams,
    mut options: RequestOptions,
) -> RequestOptions {
    insert_query_if_absent(&mut options, "after", params.after);
    insert_query_if_absent(
        &mut options,
        "limit",
        params.limit.map(|value| value.to_string()),
    );
    insert_query_if_absent(
        &mut options,
        "order",
        params.order.map(order_as_query_value),
    );
    options
}

/// 仅在 key 尚不存在时写入查询参数（避免覆盖调用方显式设置的值）。
fn insert_query_if_absent(options: &mut RequestOptions, key: &str, value: Option<String>) {
    let Some(value) = value else {
        return;
    };

    options.extra_query.entry(key.to_string()).or_insert(value);
}

fn order_as_query_value(order: ChatListOrder) -> String {
    match order {
        ChatListOrder::Asc => "asc".to_string(),
        ChatListOrder::Desc => "desc".to_string(),
    }
}

/// 构建 Chat Completion 资源路径，并对 completion_id 做路径安全编码。
fn completion_path(completion_id: &str, suffix: Option<&str>) -> Result<String> {
    if completion_id.is_empty() {
        return Err(Error::Config("completion_id must not be empty".to_string()));
    }

    let completion_id = encode_path_segment(completion_id);
    Ok(match suffix {
        Some(suffix) => format!("/chat/completions/{completion_id}/{suffix}"),
        None => format!("/chat/completions/{completion_id}"),
    })
}
