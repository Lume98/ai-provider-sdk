//! Chat 领域的数据模型。
//!
//! 包含 Chat Completions API 的请求参数、消息结构与响应壳类型。
//! 对应 OpenAI API 的 `/chat/completions` 端点。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::pagination::CursorPageItem;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
/// 聊天消息角色。序列化为 API 要求的小写字符串。
///
/// - `System` → `"system"`：系统级指令（设定模型行为）
/// - `Developer` → `"developer"`：开发者指令（新式系统提示）
/// - `User` → `"user"`：用户输入
/// - `Assistant` → `"assistant"`：模型回复
/// - `Tool` → `"tool"`：工具调用结果
pub enum ChatRole {
    System,
    Developer,
    User,
    Assistant,
    Tool,
}

/// 聊天消息结构。
///
/// `content` 使用 `serde_json::Value` 以兼容：
/// - 纯文本：`"hello"`
/// - 多模态块：`[{"type": "text", "text": "..."}, {"type": "image_url", ...}]`
/// - 未来新增的结构化内容格式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    /// 消息角色。
    pub role: ChatRole,
    /// 消息内容。使用 `Value` 兼容文本和多模态块格式。
    pub content: Value,
    /// 前向兼容扩展字段（如 `name`、`tool_calls`、`tool_call_id` 等）。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl ChatMessage {
    /// 构造 `user` 角色消息（快捷方法）。
    ///
    /// ```no_run
    /// use ai_provider_sdk::ChatMessage;
    /// let msg = ChatMessage::user("Hello!");
    /// ```
    pub fn user(content: impl Into<Value>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
            extra: HashMap::new(),
        }
    }

    /// 构造 `developer` 角色消息（快捷方法）。
    ///
    /// 用于新式系统提示（替代传统的 `system` 角色）。
    pub fn developer(content: impl Into<Value>) -> Self {
        Self {
            role: ChatRole::Developer,
            content: content.into(),
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Default)]
/// Chat Completions 创建参数。
///
/// 兼容性说明：同时保留 `max_tokens` 与 `max_completion_tokens` 以覆盖不同模型族。
/// 新模型推荐使用 `max_completion_tokens`。
pub struct ChatCompletionCreateParams {
    /// 目标模型 ID（如 `"gpt-4.1-mini"`、`"o3"`）。
    pub model: String,
    /// 对话消息序列（至少包含一条消息）。
    pub messages: Vec<ChatMessage>,
    /// 采样温度（0.0 ~ 2.0）。越高输出越随机，越低越确定。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// 核采样阈值。与 `temperature` 二选一。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// 生成token上限（新式，推荐）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,
    /// 生成token上限（旧式，兼容部分模型）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// 流式选项（如 `{"include_usage": true}` 以获取 token 使用统计）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<Value>,
    /// 是否存储响应用于后续 distillation / evals。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    /// 前向兼容扩展字段（如 `tools`、`response_format`、`seed` 等）。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl ChatCompletionCreateParams {
    /// 创建参数对象（必填字段为 `model` 和 `messages`）。
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            ..Self::default()
        }
    }
}

/// Chat Completions 列表分页参数。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatCompletionListParams {
    /// 分页游标：从该 completion ID 之后开始返回。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// 单页数量上限。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// metadata 过滤条件。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// 模型 ID 过滤条件。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// 创建时间排序方向。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<ChatListOrder>,
}

impl ChatCompletionListParams {
    /// 创建默认（无过滤）分页参数。
    pub fn new() -> Self {
        Self {
            after: None,
            limit: None,
            metadata: None,
            model: None,
            order: None,
        }
    }
}

impl Default for ChatCompletionListParams {
    fn default() -> Self {
        Self::new()
    }
}

/// Stored Chat Completion metadata 更新参数。
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct ChatCompletionUpdateParams {
    /// 需要替换或设置的 metadata。
    pub metadata: HashMap<String, String>,
}

impl ChatCompletionUpdateParams {
    /// 创建 metadata 更新参数。
    pub fn new(metadata: HashMap<String, String>) -> Self {
        Self { metadata }
    }
}

/// Stored Chat Completion 消息列表分页参数。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatCompletionMessageListParams {
    /// 分页游标：从该 message ID 之后开始返回。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// 单页数量上限。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// 创建时间排序方向。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<ChatListOrder>,
}

impl ChatCompletionMessageListParams {
    /// 创建默认（无过滤）分页参数。
    pub fn new() -> Self {
        Self {
            after: None,
            limit: None,
            order: None,
        }
    }
}

impl Default for ChatCompletionMessageListParams {
    fn default() -> Self {
        Self::new()
    }
}

/// Chat 列表排序方向枚举。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChatListOrder {
    /// 升序（最早的在前）。
    Asc,
    /// 降序（最新的在前）。
    Desc,
}

/// Chat Completion 完整响应。
///
/// 注意：仅强类型解析 `id` 字段，其余字段（如 `choices`、`usage`）
/// 存放在 `extra` 中，调用方可通过 `extra["choices"]` 访问。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ChatCompletion {
    /// 响应唯一 ID（如 `"chatcmpl-abc123"`）。
    pub id: String,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl CursorPageItem for ChatCompletion {
    fn id(&self) -> Option<&str> {
        Some(&self.id)
    }
}

/// Chat Completion 流式增量块。
///
/// 与 `ChatCompletion` 结构类似，但 `extra` 中包含的是增量数据
/// （如 `choices[0].delta.content`）。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ChatCompletionChunk {
    /// 块唯一 ID（同一流式响应的所有块共享相同 ID）。
    pub id: String,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Stored Chat Completion 删除确认响应。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ChatCompletionDeleted {
    /// 被删除的 Chat Completion ID。
    pub id: String,
    /// 是否删除成功。
    pub deleted: bool,
    /// 对象类型标识。
    #[serde(default)]
    pub object: Option<String>,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Stored Chat Completion 的消息对象。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ChatCompletionStoreMessage {
    /// 消息唯一 ID。
    pub id: String,
    /// 消息角色。
    #[serde(default)]
    pub role: Option<ChatRole>,
    /// 消息内容，保持 JSON 形态兼容多模态与未来结构。
    #[serde(default)]
    pub content: Option<Value>,
    /// 对象类型标识。
    #[serde(default)]
    pub object: Option<String>,
    /// 创建时间（Unix 时间戳）。
    #[serde(default)]
    pub created_at: Option<u64>,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl CursorPageItem for ChatCompletionStoreMessage {
    fn id(&self) -> Option<&str> {
        Some(&self.id)
    }
}
