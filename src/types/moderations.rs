//! Moderations 领域的数据模型。
//!
//! 包含输入联合类型、审核结果结构、类别判定与分数等类型。
//! 对应 OpenAI API 的 `/moderations` 端点。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 内容审核创建参数。
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ModerationCreateParams {
    /// 待审核输入（支持文本、批量文本和多模态输入）。
    pub input: ModerationInput,
    /// 可选模型 ID（如 `"omni-moderation-latest"`）。
    /// 为空时由服务端选择默认模型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl ModerationCreateParams {
    /// 以输入内容创建审核参数。
    pub fn new(input: impl Into<ModerationInput>) -> Self {
        Self {
            input: input.into(),
            model: None,
            extra: HashMap::new(),
        }
    }

    /// 指定审核模型（builder 模式）。
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}

/// 审核输入联合类型。
///
/// 设计目标：在静态类型中覆盖文本、批量文本与多模态输入。
/// 使用 `#[serde(untagged)]` 让调用方按需选择。
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ModerationInput {
    /// 单条文本。
    Text(String),
    /// 多条文本（批量审核）。
    Texts(Vec<String>),
    /// 多模态输入项（文本 + 图片 URL）。
    Items(Vec<ModerationInputItem>),
}

// 从常见 Rust 类型自动转换
impl From<&str> for ModerationInput {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<String> for ModerationInput {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<Vec<String>> for ModerationInput {
    fn from(value: Vec<String>) -> Self {
        Self::Texts(value)
    }
}

impl From<Vec<ModerationInputItem>> for ModerationInput {
    fn from(value: Vec<ModerationInputItem>) -> Self {
        Self::Items(value)
    }
}

/// 多模态审核输入项。
///
/// 使用 `#[serde(tag = "type")]` 做内部标签序列化，
/// 对应 API 的 `{"type": "text", ...}` / `{"type": "image_url", ...}` 格式。
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModerationInputItem {
    /// 文本输入项。
    Text { text: String },
    /// 图片 URL 输入项。
    ImageUrl { image_url: ModerationImageUrl },
}

impl ModerationInputItem {
    /// 创建文本输入项（快捷方法）。
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    /// 创建图片 URL 输入项（快捷方法）。
    pub fn image_url(url: impl Into<String>) -> Self {
        Self::ImageUrl {
            image_url: ModerationImageUrl { url: url.into() },
        }
    }
}

/// 图片 URL 输入结构。
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ModerationImageUrl {
    /// 图片 URL（支持 `https://` 和 `data:image/...;base64,...` 格式）。
    pub url: String,
}

/// 审核响应体。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CreateModerationResponse {
    /// 响应唯一 ID（如 `"modr-abc123"`）。
    pub id: String,
    /// 使用的审核模型 ID。
    pub model: String,
    /// 审核结果列表（每条输入对应一个结果）。
    pub results: Vec<ModerationResult>,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// 单条输入的审核结果。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ModerationResult {
    /// 是否被标记为违规（任一类别为 `true` 时此字段为 `true`）。
    pub flagged: bool,
    /// 各审核类别的二值判定结果。
    pub categories: ModerationCategories,
    /// 各审核类别的置信度分数（0.0 ~ 1.0）。
    pub category_scores: ModerationCategoryScores,
    /// 各审核类别应用于的输入类型（文本/图片）。
    #[serde(default)]
    pub category_applied_input_types: Option<ModerationCategoryAppliedInputTypes>,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// 审核类别二值判定结果。
///
/// 每个字段对应一种内容策略类别，`true` 表示该类别被触发。
/// 使用 `#[serde(default)]` 确保新增类别不会导致反序列化失败。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct ModerationCategories {
    #[serde(default)]
    pub sexual: Option<bool>,
    #[serde(default)]
    pub hate: Option<bool>,
    #[serde(default)]
    pub harassment: Option<bool>,
    #[serde(default)]
    #[serde(rename = "self-harm")]
    pub self_harm: Option<bool>,
    #[serde(default)]
    #[serde(rename = "sexual/minors")]
    pub sexual_minors: Option<bool>,
    #[serde(default)]
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: Option<bool>,
    #[serde(default)]
    #[serde(rename = "violence/graphic")]
    pub violence_graphic: Option<bool>,
    #[serde(default)]
    #[serde(rename = "self-harm/intent")]
    pub self_harm_intent: Option<bool>,
    #[serde(default)]
    #[serde(rename = "self-harm/instructions")]
    pub self_harm_instructions: Option<bool>,
    #[serde(default)]
    #[serde(rename = "harassment/threatening")]
    pub harassment_threatening: Option<bool>,
    #[serde(default)]
    pub violence: Option<bool>,
    #[serde(default)]
    pub illicit: Option<bool>,
    #[serde(default)]
    #[serde(rename = "illicit/violent")]
    pub illicit_violent: Option<bool>,
    /// 前向兼容扩展字段（捕获未来新增的审核类别）。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// 审核类别置信度分数。
///
/// 每个字段对应 [`ModerationCategories`] 中的同名类别，
/// 值为 0.0 ~ 1.0 的浮点数，越高表示越可能触发该类别。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct ModerationCategoryScores {
    #[serde(default)]
    pub sexual: Option<f64>,
    #[serde(default)]
    pub hate: Option<f64>,
    #[serde(default)]
    pub harassment: Option<f64>,
    #[serde(default)]
    #[serde(rename = "self-harm")]
    pub self_harm: Option<f64>,
    #[serde(default)]
    #[serde(rename = "sexual/minors")]
    pub sexual_minors: Option<f64>,
    #[serde(default)]
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: Option<f64>,
    #[serde(default)]
    #[serde(rename = "violence/graphic")]
    pub violence_graphic: Option<f64>,
    #[serde(default)]
    #[serde(rename = "self-harm/intent")]
    pub self_harm_intent: Option<f64>,
    #[serde(default)]
    #[serde(rename = "self-harm/instructions")]
    pub self_harm_instructions: Option<f64>,
    #[serde(default)]
    #[serde(rename = "harassment/threatening")]
    pub harassment_threatening: Option<f64>,
    #[serde(default)]
    pub violence: Option<f64>,
    #[serde(default)]
    pub illicit: Option<f64>,
    #[serde(default)]
    #[serde(rename = "illicit/violent")]
    pub illicit_violent: Option<f64>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// 各审核类别应用于的输入类型映射。
///
/// 每个字段指示该类别基于哪些输入类型进行判定。
/// 例如 `violence_graphic` 可能只应用于 `image` 输入。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct ModerationCategoryAppliedInputTypes {
    #[serde(default)]
    pub sexual: Option<Vec<ModerationAppliedInputType>>,
    #[serde(default)]
    pub hate: Option<Vec<ModerationAppliedInputType>>,
    #[serde(default)]
    pub harassment: Option<Vec<ModerationAppliedInputType>>,
    #[serde(default)]
    #[serde(rename = "self-harm")]
    pub self_harm: Option<Vec<ModerationAppliedInputType>>,
    #[serde(default)]
    #[serde(rename = "sexual/minors")]
    pub sexual_minors: Option<Vec<ModerationAppliedInputType>>,
    #[serde(default)]
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: Option<Vec<ModerationAppliedInputType>>,
    #[serde(default)]
    #[serde(rename = "violence/graphic")]
    pub violence_graphic: Option<Vec<ModerationAppliedInputType>>,
    #[serde(default)]
    #[serde(rename = "self-harm/intent")]
    pub self_harm_intent: Option<Vec<ModerationAppliedInputType>>,
    #[serde(default)]
    #[serde(rename = "self-harm/instructions")]
    pub self_harm_instructions: Option<Vec<ModerationAppliedInputType>>,
    #[serde(default)]
    #[serde(rename = "harassment/threatening")]
    pub harassment_threatening: Option<Vec<ModerationAppliedInputType>>,
    #[serde(default)]
    pub violence: Option<Vec<ModerationAppliedInputType>>,
    #[serde(default)]
    pub illicit: Option<Vec<ModerationAppliedInputType>>,
    #[serde(default)]
    #[serde(rename = "illicit/violent")]
    pub illicit_violent: Option<Vec<ModerationAppliedInputType>>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// 审核类别应用于的输入类型。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModerationAppliedInputType {
    /// 文本输入。
    Text,
    /// 图片输入。
    Image,
}
