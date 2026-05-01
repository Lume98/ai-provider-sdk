use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ModerationCreateParams {
    pub input: ModerationInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl ModerationCreateParams {
    pub fn new(input: impl Into<ModerationInput>) -> Self {
        Self {
            input: input.into(),
            model: None,
            extra: HashMap::new(),
        }
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ModerationInput {
    Text(String),
    Texts(Vec<String>),
    Items(Vec<ModerationInputItem>),
}

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

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModerationInputItem {
    Text { text: String },
    ImageUrl { image_url: ModerationImageUrl },
}

impl ModerationInputItem {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    pub fn image_url(url: impl Into<String>) -> Self {
        Self::ImageUrl {
            image_url: ModerationImageUrl { url: url.into() },
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ModerationImageUrl {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CreateModerationResponse {
    pub id: String,
    pub model: String,
    pub results: Vec<ModerationResult>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ModerationResult {
    pub flagged: bool,
    pub categories: ModerationCategories,
    pub category_scores: ModerationCategoryScores,
    #[serde(default)]
    pub category_applied_input_types: Option<ModerationCategoryAppliedInputTypes>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

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
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModerationAppliedInputType {
    Text,
    Image,
}
