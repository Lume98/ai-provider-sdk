//! Files 领域的数据模型。包含上传参数、分页参数与文件实体。

use std::collections::HashMap;
use std::path::Path;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, Result};
use crate::pagination::CursorPageItem;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// 文件用途枚举。按 API wire 值做显式 `serde` rename。
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
    /// 上传时使用的文件名（会传给 multipart part 的 filename）。
    pub file_name: String,
    /// 文件原始字节。
    pub bytes: Bytes,
    /// 可选 MIME 类型。为空时不显式设置 content-type。
    pub mime_type: Option<String>,
}

impl UploadFile {
    /// 由内存字节创建上传文件对象。
    pub fn from_bytes(file_name: impl Into<String>, bytes: impl Into<Bytes>) -> Self {
        Self {
            file_name: file_name.into(),
            bytes: bytes.into(),
            mime_type: None,
        }
    }

    /// 从本地路径异步读取文件并创建上传对象。
    ///
    /// 边界条件：文件名必须是有效 UTF-8，否则返回配置错误。
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
    /// 必填上传文件内容。
    pub file: UploadFile,
    /// 文件用途（影响服务端校验和后续可用场景）。
    pub purpose: FilePurpose,
    /// 可选过期策略。
    pub expires_after: Option<ExpiresAfter>,
    /// 前向兼容扩展字段。
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
    /// 过期锚点（由服务端定义有效值集合）。
    pub anchor: String,
    /// 相对锚点的秒数。
    pub seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileListParams {
    /// 分页游标：从该 ID 之后开始返回。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// 单页数量上限。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// 创建时间排序方向。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<ListOrder>,
    /// 用途过滤条件（字符串透传）。
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
    /// 文件唯一 ID。
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
