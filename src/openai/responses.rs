use std::collections::HashMap;

use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    DeletedObject, JsonObject, RequestOptions, ResponsesResource, TypedSseStream,
    chat::ContentPart, core::path_segment,
};
use crate::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResponseCreateParams {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<ResponseInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ResponseTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[serde(default)]
    pub stream: bool,
    #[serde(flatten)]
    pub extra: JsonObject,
}

impl ResponseCreateParams {
    pub fn new(model: impl Into<String>, input: impl Into<ResponseInput>) -> Self {
        Self {
            model: model.into(),
            input: Some(input.into()),
            instructions: None,
            previous_response_id: None,
            tools: None,
            tool_choice: None,
            temperature: None,
            top_p: None,
            max_output_tokens: None,
            reasoning: None,
            text: None,
            metadata: None,
            store: None,
            stream: false,
            extra: JsonObject::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseInput {
    Text(String),
    Items(Vec<ResponseInputItem>),
}

impl From<&str> for ResponseInput {
    fn from(value: &str) -> Self {
        Self::Text(value.into())
    }
}

impl From<String> for ResponseInput {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ResponseInputItem {
    Message {
        role: String,
        content: Vec<ContentPart>,
    },
    FunctionCallOutput {
        call_id: String,
        output: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ResponseTool {
    Function {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },
    WebSearchPreview,
    FileSearch {
        vector_store_ids: Vec<String>,
        #[serde(flatten)]
        extra: JsonObject,
    },
    ComputerUsePreview {
        #[serde(flatten)]
        extra: JsonObject,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResponseObject {
    pub id: String,
    pub object: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub output: Vec<ResponseOutputItem>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ResponseOutputItem {
    Message {
        id: Option<String>,
        role: String,
        content: Vec<ResponseOutputContent>,
        #[serde(flatten)]
        extra: JsonObject,
    },
    FunctionCall {
        id: Option<String>,
        call_id: String,
        name: String,
        arguments: String,
        #[serde(flatten)]
        extra: JsonObject,
    },
    #[serde(other)]
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ResponseOutputContent {
    OutputText {
        text: String,
        #[serde(default)]
        annotations: Vec<Value>,
    },
    Refusal {
        refusal: String,
    },
    #[serde(other)]
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResponseEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub sequence_number: Option<u64>,
    #[serde(flatten)]
    pub data: JsonObject,
}

impl ResponsesResource {
    pub async fn create(&self, params: &ResponseCreateParams) -> Result<ResponseObject, Error> {
        let mut params = params.clone();
        params.stream = false;
        self.core
            .json_value(
                Method::POST,
                "/responses",
                Option::<&()>::None,
                Some(&params),
                RequestOptions::default(),
            )
            .await
    }

    pub async fn stream(
        &self,
        params: &ResponseCreateParams,
    ) -> Result<TypedSseStream<ResponseEvent>, Error> {
        let mut params = params.clone();
        params.stream = true;
        self.core
            .stream(
                Method::POST,
                "/responses",
                Option::<&()>::None,
                Some(&params),
                RequestOptions::default(),
            )
            .await
    }

    pub async fn retrieve(&self, response_id: &str) -> Result<ResponseObject, Error> {
        self.core
            .json_value(
                Method::GET,
                &format!("/responses/{}", path_segment(response_id)),
                Option::<&()>::None,
                Option::<&()>::None,
                RequestOptions::default(),
            )
            .await
    }

    pub async fn delete(&self, response_id: &str) -> Result<DeletedObject, Error> {
        self.core
            .json_value(
                Method::DELETE,
                &format!("/responses/{}", path_segment(response_id)),
                Option::<&()>::None,
                Option::<&()>::None,
                RequestOptions::default(),
            )
            .await
    }
}
