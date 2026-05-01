//! Models 领域的数据模型。描述模型对象与模型列表结构。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
/// 模型对象。未知字段进入 `extra`，防止服务端增量字段导致反序列化失败。
pub struct Model {
    pub id: String,
    #[serde(default)]
    pub object: Option<String>,
    #[serde(default)]
    pub created: Option<u64>,
    #[serde(default)]
    pub owned_by: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ModelList {
    #[serde(default)]
    pub object: Option<String>,
    pub data: Vec<Model>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}
