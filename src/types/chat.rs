use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    Developer,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: Value,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl ChatMessage {
    pub fn user(content: impl Into<Value>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
            extra: HashMap::new(),
        }
    }

    pub fn developer(content: impl Into<Value>) -> Self {
        Self {
            role: ChatRole::Developer,
            content: content.into(),
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ChatCompletionCreateParams {
    pub model: String,
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
