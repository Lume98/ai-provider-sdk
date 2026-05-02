//! Files 领域的数据模型。
//!
//! 包含文件上传参数、分页参数、文件实体与删除确认等类型。
//! 对应 OpenAI API 的 `/files` 端点族。

use std::collections::HashMap;
use std::path::Path;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, Result};
use crate::pagination::CursorPageItem;

/// 文件用途枚举。
///
/// 用途影响服务端对文件格式的校验和后续可用场景：
/// - `FineTune`：用于 fine-tuning 训练数据。
/// - `Batch` / `BatchOutput`：用于 Batch API 的输入和输出。
/// - `Assistants` / `AssistantsOutput`：用于 Assistants API。
/// - `Vision`：用于图片上传。
/// - `UserData`：通用用户数据。
/// - `Evals`：用于模型评估。
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

/// 上传文件描述。
///
/// 封装文件名、字节内容和可选 MIME 类型，
/// 用于构建 multipart 上传请求。
#[derive(Debug, Clone)]
pub struct UploadFile {
    /// 上传时使用的文件名（会传给 multipart part 的 filename）。
    pub file_name: String,
    /// 文件原始字节。
    pub bytes: Bytes,
    /// 可选 MIME 类型。为空时不显式设置 content-type，
    /// 由服务端或下游自行推断。
    pub mime_type: Option<String>,
}

impl UploadFile {
    /// 由内存字节创建上传文件对象。
    ///
    /// ```no_run
    /// use ai_provider_sdk::UploadFile;
    /// use bytes::Bytes;
    ///
    /// let file = UploadFile::from_bytes("train.jsonl", Bytes::from_static(b"[...]"));
    /// ```
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

/// 文件上传创建参数。
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
    /// 创建上传参数（必填字段为 `file` 和 `purpose`）。
    pub fn new(file: UploadFile, purpose: FilePurpose) -> Self {
        Self {
            file,
            purpose,
            expires_after: None,
            extra: HashMap::new(),
        }
    }
}

/// 文件过期策略。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExpiresAfter {
    /// 过期锚点（由服务端定义有效值集合，如 `"last_active_time"`）。
    pub anchor: String,
    /// 相对锚点的秒数。
    pub seconds: u32,
}

/// 文件列表分页参数。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileListParams {
    /// 分页游标：从该 ID 之后开始返回。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// 单页数量上限（默认 20，最大 10000）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// 创建时间排序方向。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<ListOrder>,
    /// 用途过滤条件（按 `FilePurpose` 的 wire 值字符串过滤）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
}

impl FileListParams {
    /// 创建默认（无过滤）分页参数。
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

/// 排序方向枚举。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ListOrder {
    /// 升序（最早的在前）。
    Asc,
    /// 降序（最新的在前）。
    Desc,
}

/// 文件对象元信息。
///
/// 服务端在文件上传、列表、详情查询时返回此结构。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct FileObject {
    /// 文件唯一 ID（如 `"file-abc123"`）。
    pub id: String,
    /// 文件大小（字节）。
    #[serde(default)]
    pub bytes: Option<u64>,
    /// 创建时间（Unix 时间戳）。
    #[serde(default)]
    pub created_at: Option<u64>,
    /// 原始上传文件名。
    #[serde(default)]
    pub filename: Option<String>,
    /// 对象类型标识（通常为 `"file"`）。
    #[serde(default)]
    pub object: Option<String>,
    /// 文件用途。
    #[serde(default)]
    pub purpose: Option<FilePurpose>,
    /// 文件状态（如 `"uploaded"`、`"processed"`、`"error"`）。
    #[serde(default)]
    pub status: Option<String>,
    /// 过期时间（Unix 时间戳，仅当设置了过期策略时存在）。
    #[serde(default)]
    pub expires_at: Option<u64>,
    /// 状态详情（如错误信息）。
    #[serde(default)]
    pub status_details: Option<String>,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl CursorPageItem for FileObject {
    fn id(&self) -> Option<&str> {
        Some(&self.id)
    }
}

/// 文件删除确认响应。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct FileDeleted {
    /// 被删除的文件 ID。
    pub id: String,
    /// 是否删除成功。
    pub deleted: bool,
    /// 对象类型标识（通常为 `"file"`）。
    #[serde(default)]
    pub object: Option<String>,
    /// 前向兼容扩展字段。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}
