use std::collections::HashMap;
use std::path::Path;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, Result};
use crate::pagination::CursorPageItem;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FilePurpose {
    #[serde(rename = "assistants")]
    Assistants,
    #[serde(rename = "assistants_output")]
    AssistantsOutput,
    #[serde(rename = "batch")]
    Batch,
    #[serde(rename = "batch_output")]
    BatchOutput,
    #[serde(rename = "fine-tune")]
    FineTune,
    #[serde(rename = "fine-tune-results")]
    FineTuneResults,
    #[serde(rename = "vision")]
    Vision,
    #[serde(rename = "user_data")]
    UserData,
    #[serde(rename = "evals")]
    Evals,
}

#[derive(Debug, Clone)]
pub struct UploadFile {
    pub file_name: String,
    pub bytes: Bytes,
    pub mime_type: Option<String>,
}

impl UploadFile {
    pub fn from_bytes(file_name: impl Into<String>, bytes: impl Into<Bytes>) -> Self {
        Self {
            file_name: file_name.into(),
            bytes: bytes.into(),
            mime_type: None,
        }
    }

    pub async fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let bytes = tokio::fs::read(path).await?;
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| {
                Error::Config("file path must have a valid UTF-8 file name".to_string())
            })?;

        Ok(Self {
            file_name: file_name.to_string(),
            bytes: Bytes::from(bytes),
            mime_type: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FileCreateParams {
    pub file: UploadFile,
    pub purpose: FilePurpose,
    pub expires_after: Option<ExpiresAfter>,
    pub extra: HashMap<String, Value>,
}

impl FileCreateParams {
    pub fn new(file: UploadFile, purpose: FilePurpose) -> Self {
        Self {
            file,
            purpose,
            expires_after: None,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExpiresAfter {
    pub anchor: String,
    pub seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<ListOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
}

impl FileListParams {
    pub fn new() -> Self {
        Self {
            after: None,
            limit: None,
            order: None,
            purpose: None,
        }
    }
}

impl Default for FileListParams {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ListOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct FileObject {
    pub id: String,
    #[serde(default)]
    pub bytes: Option<u64>,
    #[serde(default)]
    pub created_at: Option<u64>,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub object: Option<String>,
    #[serde(default)]
    pub purpose: Option<FilePurpose>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub expires_at: Option<u64>,
    #[serde(default)]
    pub status_details: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl CursorPageItem for FileObject {
    fn id(&self) -> Option<&str> {
        Some(&self.id)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct FileDeleted {
    pub id: String,
    pub deleted: bool,
    #[serde(default)]
    pub object: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}
