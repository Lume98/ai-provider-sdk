//! Chat 领域的数据模型。包含请求参数、消息结构与响应壳类型。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
/// 聊天消息角色。序列化为 API 要求的小写字符串。
pub enum ChatRole {
    System,
    Developer,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    /// 消息角色。
    pub role: ChatRole,
    /// 消息内容。
    ///
    /// 使用 `Value` 是为了兼容文本、多模态块以及未来扩展结构。
    pub content: Value,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl ChatMessage {
    /// 构造 `user` 角色消息。
    pub fn user(content: impl Into<Value>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
            extra: HashMap::new(),
        }
    }

    /// 构造 `developer` 角色消息。
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
/// 兼容性说明：保留 `max_tokens` 与 `max_completion_tokens` 以覆盖不同模型族。
pub struct ChatCompletionCreateParams {
    /// 目标模型 ID。
    pub model: String,
    /// 对话消息序列。
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl ChatCompletionCreateParams {
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ChatCompletion {
    pub id: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ChatCompletionChunk {
    pub id: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}
