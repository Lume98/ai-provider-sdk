//! Embeddings 领域的数据模型。
//!
//! 包含向量生成的请求参数、输入联合类型、编码格式与响应结构。
//! 对应 OpenAI API 的 `/embeddings` 端点。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Embeddings 创建参数。
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct EmbeddingCreateParams {
    /// 目标模型 ID（如 `"text-embedding-3-small"`、`"text-embedding-3-large"`）。
    pub model: String,
    /// 向量输入（文本或 token 形式）。
    pub input: EmbeddingInput,
    /// 输出向量维度（仅部分模型支持，如 `text-embedding-3-small`）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
    /// 输出编码格式。为空时由资源层自动填充为 `Float`。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<EncodingFormat>,
    /// 用户标识，用于滥用监控。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl EmbeddingCreateParams {
    /// 构造 embedding 创建参数。
    ///
    /// ```no_run
    /// use ai_provider_sdk::{EmbeddingCreateParams, EmbeddingInput};
    ///
    /// // 单文本
    /// let params = EmbeddingCreateParams::new("text-embedding-3-small", "hello");
    ///
    /// // 多文本
    /// let params = EmbeddingCreateParams::new(
    ///     "text-embedding-3-small",
    ///     vec!["hello".to_string(), "world".to_string()],
    /// );
    /// ```
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

/// 向量输入联合类型。
///
/// 支持四种输入协议，通过 `#[serde(untagged)]` 让调用方按需选择：
/// - 单个文本字符串
/// - 多个文本字符串数组
/// - 单个 token ID 数组
/// - 多个 token ID 数组（批量）
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum EmbeddingInput {
    /// 单个文本字符串。
    Text(String),
    /// 多个文本字符串。
    Texts(Vec<String>),
    /// 单个 token ID 数组。
    Tokens(Vec<u32>),
    /// 多个 token ID 数组（批量嵌入）。
    TokenBatches(Vec<Vec<u32>>),
}

// 从常见 Rust 类型自动转换，简化调用方代码
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

/// 输出向量编码格式。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EncodingFormat {
    /// 浮点数组（默认）。
    Float,
    /// Base64 编码字符串（更紧凑，适合大量向量传输）。
    Base64,
}

/// Embeddings API 响应体。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CreateEmbeddingResponse {
    /// 响应对象类型标识（通常为 `"list"`）。
    #[serde(default)]
    pub object: Option<String>,
    /// 嵌入向量列表。
    pub data: Vec<Embedding>,
    /// 使用的模型 ID。
    #[serde(default)]
    pub model: Option<String>,
    /// token 使用统计。
    #[serde(default)]
    pub usage: Option<EmbeddingUsage>,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// 单个嵌入向量结果。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Embedding {
    /// 对象类型标识（通常为 `"embedding"`）。
    #[serde(default)]
    pub object: Option<String>,
    /// 输入中的索引位置。
    pub index: u32,
    /// 嵌入向量数据（浮点数组或 base64 字符串）。
    pub embedding: EmbeddingVector,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// 嵌入向量数据格式。
///
/// 根据 `encoding_format` 参数，服务端返回不同格式：
/// - `Float`：`[0.1, 0.2, ...]`（浮点数组）
/// - `Base64`：`"AQID..."`（base64 编码字符串）
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum EmbeddingVector {
    /// 浮点向量。
    Float(Vec<f64>),
    /// Base64 编码向量。
    Base64(String),
}

/// Embeddings API token 使用统计。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EmbeddingUsage {
    /// 输入 token 数。
    pub prompt_tokens: u32,
    /// 总 token 数（含输入和输出）。
    pub total_tokens: u32,
}
