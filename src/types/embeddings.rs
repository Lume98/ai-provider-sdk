//! Embeddings 领域的数据模型。包含输入联合类型、编码格式与响应结构。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct EmbeddingCreateParams {
    /// 目标模型 ID。
    pub model: String,
    /// 向量输入（文本或 token 形式）。
    pub input: EmbeddingInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<EncodingFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl EmbeddingCreateParams {
    /// 构造 embedding 创建参数。
    pub fn new(model: impl Into<String>, input: impl Into<EmbeddingInput>) -> Self {
        Self {
            model: model.into(),
            input: input.into(),
            dimensions: None,
            encoding_format: None,
            user: None,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
/// 向量输入联合类型，支持文本与 token 两套输入协议。
pub enum EmbeddingInput {
    Text(String),
    Texts(Vec<String>),
    Tokens(Vec<u32>),
    TokenBatches(Vec<Vec<u32>>),
}

impl From<&str> for EmbeddingInput {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<String> for EmbeddingInput {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<Vec<String>> for EmbeddingInput {
    fn from(value: Vec<String>) -> Self {
        Self::Texts(value)
    }
}

impl From<Vec<u32>> for EmbeddingInput {
    fn from(value: Vec<u32>) -> Self {
        Self::Tokens(value)
    }
}

impl From<Vec<Vec<u32>>> for EmbeddingInput {
    fn from(value: Vec<Vec<u32>>) -> Self {
        Self::TokenBatches(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EncodingFormat {
    /// 返回浮点数组。
    Float,
    /// 返回 base64 编码字符串。
    Base64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CreateEmbeddingResponse {
    #[serde(default)]
    pub object: Option<String>,
    pub data: Vec<Embedding>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub usage: Option<EmbeddingUsage>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Embedding {
    #[serde(default)]
    pub object: Option<String>,
    pub index: u32,
    pub embedding: EmbeddingVector,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum EmbeddingVector {
    /// 浮点向量。
    Float(Vec<f64>),
    /// base64 编码向量。
    Base64(String),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EmbeddingUsage {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}
