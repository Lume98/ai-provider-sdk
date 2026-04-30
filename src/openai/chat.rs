use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{ChatCompletionsResource, JsonObject, RequestOptions, TypedSseStream, generic::Usage};
use crate::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub response_format: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ChatTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[serde(default)]
    pub stream: bool,
    #[serde(flatten)]
    pub extra: JsonObject,
}

impl ChatCompletionCreateParams {
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            temperature: None,
            top_p: None,
            max_completion_tokens: None,
            response_format: None,
            reasoning_effort: None,
            tools: None,
            tool_choice: None,
            stream: false,
            extra: JsonObject::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "role")]
#[serde(rename_all = "snake_case")]
pub enum ChatMessage {
    Developer {
        content: MessageContent,
        #[serde(flatten)]
        extra: JsonObject,
    },
    System {
        content: MessageContent,
        #[serde(flatten)]
        extra: JsonObject,
    },
    User {
        content: MessageContent,
        #[serde(flatten)]
        extra: JsonObject,
    },
    Assistant {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        content: Option<MessageContent>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ChatToolCall>>,
        #[serde(flatten)]
        extra: JsonObject,
    },
    Tool {
        content: MessageContent,
        tool_call_id: String,
        #[serde(flatten)]
        extra: JsonObject,
    },
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self::User {
            content: MessageContent::Text(content.into()),
            extra: JsonObject::new(),
        }
    }

    pub fn developer(content: impl Into<String>) -> Self {
        Self::Developer {
            content: MessageContent::Text(content.into()),
            extra: JsonObject::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
    InputAudio { input_audio: Value },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ChatTool {
    Function { function: FunctionDefinition },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatCompletion {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    #[serde(default)]
    pub usage: Option<Usage>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatResponseMessage {
    pub role: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ChatToolCall>>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    #[serde(default)]
    pub id: Option<String>,
    pub object: String,
    #[serde(default)]
    pub created: Option<i64>,
    #[serde(default)]
    pub model: Option<String>,
    pub choices: Vec<ChatChunkChoice>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatChunkChoice {
    pub index: u32,
    pub delta: ChatDelta,
    pub finish_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ChatDelta {
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ChatToolCall>>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

impl ChatCompletionsResource {
    pub async fn create(
        &self,
        params: &ChatCompletionCreateParams,
    ) -> Result<ChatCompletion, Error> {
        let mut params = params.clone();
        params.stream = false;
        self.core
            .json_value(
                Method::POST,
                "/chat/completions",
                Option::<&()>::None,
                Some(&params),
                RequestOptions::default(),
            )
            .await
    }

    pub async fn create_stream(
        &self,
        params: &ChatCompletionCreateParams,
    ) -> Result<TypedSseStream<ChatCompletionChunk>, Error> {
        let mut params = params.clone();
        params.stream = true;
        self.core
            .stream(
                Method::POST,
                "/chat/completions",
                Option::<&()>::None,
                Some(&params),
                RequestOptions::default(),
            )
            .await
    }
}
