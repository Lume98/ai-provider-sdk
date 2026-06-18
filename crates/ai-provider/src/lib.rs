use std::collections::BTreeMap;
use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub type ProviderOptions = BTreeMap<String, Value>;
pub type Headers = BTreeMap<String, String>;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("API call failed: {message}")]
    ApiCall {
        message: String,
        status: Option<u16>,
        response_body: Option<String>,
    },
    #[error("authentication failed: {0}")]
    Authentication(String),
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("failed to parse response: {0}")]
    ResponseParsing(String),
    #[error("unsupported functionality: {0}")]
    Unsupported(String),
    #[error("missing model: {0}")]
    MissingModel(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Warning {
    Unsupported {
        feature: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<String>,
    },
    Compatibility {
        feature: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<String>,
    },
    Deprecated {
        setting: String,
        message: String,
    },
    Other {
        message: String,
    },
}

impl Warning {
    pub fn unsupported(feature: impl Into<String>) -> Self {
        Self::Unsupported {
            feature: feature.into(),
            details: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMetadata {
    pub id: Option<String>,
    pub model_id: Option<String>,
    pub timestamp: Option<String>,
    pub headers: Option<Headers>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionTool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parameters: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ToolChoice {
    Auto,
    None,
    Required,
    Tool { name: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResponseFormat {
    Text,
    Json {
        #[serde(skip_serializing_if = "Option::is_none")]
        schema: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Reasoning {
    ProviderDefault,
    None,
    Minimal,
    Low,
    Medium,
    High,
    XHigh,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageMessage {
    pub role: String,
    pub content: String,
}

impl LanguageMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageModelCallOptions {
    pub prompt: Vec<LanguageMessage>,
    pub max_output_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub response_format: Option<ResponseFormat>,
    pub tools: Option<Vec<FunctionTool>>,
    pub tool_choice: Option<ToolChoice>,
    pub reasoning: Option<Reasoning>,
    pub provider_options: Option<ProviderOptions>,
    pub headers: Option<Headers>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ContentPart {
    Text {
        text: String,
    },
    Reasoning {
        text: String,
    },
    ToolCall {
        id: String,
        name: String,
        input: Value,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Error,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateResult {
    pub content: Vec<ContentPart>,
    pub finish_reason: FinishReason,
    pub usage: Usage,
    pub response: Option<ResponseMetadata>,
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum LanguageModelStreamPart {
    StreamStart {
        warnings: Vec<Warning>,
    },
    TextStart {
        id: String,
    },
    TextDelta {
        id: String,
        delta: String,
    },
    TextEnd {
        id: String,
    },
    ResponseMetadata(ResponseMetadata),
    Finish {
        usage: Usage,
        finish_reason: FinishReason,
    },
    Raw {
        value: Value,
    },
    Error {
        message: String,
    },
}

pub type LanguageModelStream =
    Pin<Box<dyn Stream<Item = Result<LanguageModelStreamPart, AiError>> + Send>>;

pub struct StreamResult {
    pub stream: LanguageModelStream,
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddingResult {
    pub embeddings: Vec<Vec<f32>>,
    pub usage: Usage,
    pub response: Option<ResponseMetadata>,
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageModelCallOptions {
    pub prompt: String,
    pub n: Option<u32>,
    pub size: Option<String>,
    pub response_format: Option<String>,
    pub provider_options: Option<ProviderOptions>,
    pub headers: Option<Headers>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageResult {
    pub images: Vec<String>,
    pub response: Option<ResponseMetadata>,
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechModelCallOptions {
    pub text: String,
    pub voice: String,
    pub format: Option<String>,
    pub instructions: Option<String>,
    pub provider_options: Option<ProviderOptions>,
    pub headers: Option<Headers>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechResult {
    pub audio: Vec<u8>,
    pub media_type: String,
    pub response: Option<ResponseMetadata>,
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionModelCallOptions {
    pub audio: Vec<u8>,
    pub file_name: String,
    pub media_type: Option<String>,
    pub language: Option<String>,
    pub prompt: Option<String>,
    pub provider_options: Option<ProviderOptions>,
    pub headers: Option<Headers>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionResult {
    pub text: String,
    pub response: Option<ResponseMetadata>,
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RerankingModelCallOptions {
    pub query: String,
    pub documents: Vec<String>,
    pub top_n: Option<u32>,
    pub provider_options: Option<ProviderOptions>,
    pub headers: Option<Headers>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RerankingResult {
    pub rankings: Vec<RerankingItem>,
    pub usage: Usage,
    pub response: Option<ResponseMetadata>,
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RerankingItem {
    pub index: usize,
    pub score: f32,
}

#[async_trait]
pub trait LanguageModel: Send + Sync {
    fn provider(&self) -> &str;
    fn model_id(&self) -> &str;
    async fn generate(&self, options: LanguageModelCallOptions) -> Result<GenerateResult, AiError>;
    async fn stream(&self, options: LanguageModelCallOptions) -> Result<StreamResult, AiError>;
}

#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    fn provider(&self) -> &str;
    fn model_id(&self) -> &str;
    async fn embed(&self, values: Vec<String>) -> Result<EmbeddingResult, AiError>;
}

#[async_trait]
pub trait ImageModel: Send + Sync {
    fn provider(&self) -> &str;
    fn model_id(&self) -> &str;
    async fn generate_image(&self, options: ImageModelCallOptions) -> Result<ImageResult, AiError>;
}

#[async_trait]
pub trait SpeechModel: Send + Sync {
    fn provider(&self) -> &str;
    fn model_id(&self) -> &str;
    async fn generate_speech(
        &self,
        options: SpeechModelCallOptions,
    ) -> Result<SpeechResult, AiError>;
}

#[async_trait]
pub trait TranscriptionModel: Send + Sync {
    fn provider(&self) -> &str;
    fn model_id(&self) -> &str;
    async fn transcribe(
        &self,
        options: TranscriptionModelCallOptions,
    ) -> Result<TranscriptionResult, AiError>;
}

#[async_trait]
pub trait RerankingModel: Send + Sync {
    fn provider(&self) -> &str;
    fn model_id(&self) -> &str;
    async fn rerank(&self, options: RerankingModelCallOptions) -> Result<RerankingResult, AiError>;
}

pub trait Files: Send + Sync {}
pub trait Skills: Send + Sync {}

pub trait Provider: Send + Sync {
    fn language_model(&self, model_id: &str) -> Result<Box<dyn LanguageModel>, AiError>;
    fn embedding_model(&self, model_id: &str) -> Result<Box<dyn EmbeddingModel>, AiError>;
    fn image_model(&self, model_id: &str) -> Result<Box<dyn ImageModel>, AiError>;
    fn speech_model(&self, model_id: &str) -> Result<Box<dyn SpeechModel>, AiError>;
    fn transcription_model(&self, model_id: &str) -> Result<Box<dyn TranscriptionModel>, AiError>;
    fn reranking_model(&self, model_id: &str) -> Result<Box<dyn RerankingModel>, AiError>;
    fn files(&self) -> Result<Box<dyn Files>, AiError>;
    fn skills(&self) -> Result<Box<dyn Skills>, AiError>;
}
