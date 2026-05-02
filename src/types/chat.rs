//! Chat 领域的数据模型。
//!
//! 包含 Chat Completions API 的请求参数、消息结构与响应壳类型。
//! 对应 OpenAI API 的 `/chat/completions` 端点。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

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
