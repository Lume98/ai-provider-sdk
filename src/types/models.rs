//! Models 领域的数据模型。
//!
//! 描述模型对象、模型列表与模型删除响应结构。
//! 对应 OpenAI API 的 `/models`、`/models/{model}` 和 `DELETE /models/{model}` 端点。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 模型对象。
///
/// 未知字段进入 `extra`，防止服务端增量字段导致反序列化失败。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Model {
    /// 模型唯一 ID（如 `"gpt-4.1-mini"`、`"text-embedding-3-small"`）。
    pub id: String,
    /// 对象类型标识（通常为 `"model"`）。
    pub object: String,
    /// 创建时间（Unix 时间戳）。
    pub created: u64,
    /// 模型所有者（如 `"openai"`、`"system"`）。
    pub owned_by: String,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// 模型列表响应。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ModelList {
    /// 对象类型标识（通常为 `"list"`）。
    pub object: String,
    /// 模型对象数组。
    pub data: Vec<Model>,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// 模型删除确认响应。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ModelDeleted {
    /// 被删除的模型 ID。
    pub id: String,
    /// 是否删除成功。
    pub deleted: bool,
    /// 对象类型标识（通常为 `"model"`）。
    pub object: String,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}
