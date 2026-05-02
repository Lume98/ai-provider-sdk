//! Responses 领域的数据模型。
//!
//! 包含 Responses API 的创建参数与流式事件结构。
//! 对应 OpenAI API 的 `/responses` 端点（新一代对话 API）。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Responses 创建参数。
///
/// 支持 OpenAI Responses API 的所有参数，包括输入、指令、
/// 采样参数和元数据等。
#[derive(Debug, Clone, Serialize, Default)]
pub struct ResponseCreateParams {
    /// 目标模型 ID（如 `"gpt-4.1-mini"`、`"o3"`）。
    pub model: String,
    /// 输入内容。
    ///
    /// 支持多种 JSON 形态：
    /// - 纯文本字符串：`"hello"`
    /// - 消息数组：`[{"role": "user", "content": "hello"}]`
    /// - 结构化块：更复杂的嵌套结构
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<Value>,
    /// 系统级指令（设定模型行为）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// 最大输出 token 数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    /// 请求级元数据（键值对，会原样返回）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
    /// 采样温度（0.0 ~ 2.0）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// 核采样阈值。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// 是否存储响应用于后续分析。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    /// 流式选项（如 `{"include_usage": true}`）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<Value>,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl ResponseCreateParams {
    /// 使用模型 ID 构造参数对象。
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            ..Self::default()
        }
    }

    /// 设置输入内容（builder 模式）。
    pub fn input(mut self, input: impl Into<Value>) -> Self {
        self.input = Some(input.into());
        self
    }
}

/// Response 完整响应。
///
/// 仅强类型解析 `id` 字段，其余字段存放在 `extra` 中。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Response {
    /// 响应唯一 ID（如 `"resp_abc123"`）。
    pub id: String,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Response 流式增量事件。
///
/// 每个事件代表流式响应中的一个增量更新（如文本增量、工具调用、完成事件等）。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ResponseStreamEvent {
    /// 事件类型（如 `"response.output_text.delta"`、`"response.completed"` 等）。
    #[serde(default)]
    pub r#type: Option<String>,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}
