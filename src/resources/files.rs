//! Files 资源封装。
//!
//! 处理文件上传、分页查询、下载与删除。
//! 对应 OpenAI API 的 `/files` 端点族。
//!
//! ## 支持的操作
//!
//! | 方法               | HTTP   | 路径                     | 描述          |
//! |-------------------|--------|-------------------------|--------------|
//! | `create`          | POST   | `/files`                | 上传文件       |
//! | `retrieve`        | GET    | `/files/{file_id}`      | 查询文件元信息  |
//! | `list`            | GET    | `/files`                | 分页查询文件列表 |
//! | `delete`          | DELETE | `/files/{file_id}`      | 删除文件       |
//! | `content`         | GET    | `/files/{file_id}/content` | 下载文件内容 |
//!
//! ## 使用方式
//!
//! ```no_run
//! use ai_provider_sdk::{OpenAI, FileCreateParams, FileListParams, UploadFile, FilePurpose};
//! use bytes::Bytes;
//!
//! # async fn example(client: OpenAI) -> ai_provider_sdk::Result<()> {
//! // 上传文件
//! let file = client.files().create(FileCreateParams::new(
//!     UploadFile::from_bytes("train.jsonl", Bytes::from_static(b"[...]")),
//!     FilePurpose::FineTune,
//! )).await?;
//!
//! // 分页查询
//! let page = client.files().list_with_params(FileListParams::new()).await?;
//! for f in page.items() {
//!     println!("{} ({})", f.id, f.filename.as_deref().unwrap_or("?"));
//! }
//!
//! // 下载文件内容
//! let bytes = client.files().content(&file.id).await?;
//!
//! // 删除文件
//! client.files().delete(&file.id).await?;
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;

use async_stream::try_stream;
use bytes::Bytes;
use futures_core::Stream;
use serde_json::Value;

use crate::error::{Error, Result};
use crate::pagination::CursorPage;
use crate::path::encode_path_segment;
use crate::request_options::RequestOptions;
use crate::transport::{MultipartFile, MultipartFormData, Transport};
use crate::types::{
    FileCreateParams, FileDeleted, FileListParams, FileObject, FilePurpose, ListOrder,
};

#[derive(Clone)]
/// Files 资源入口。
///
/// 通过 `client.files()` 获取。支持 CRUD 和自动翻页流。
pub struct Files {
    transport: Arc<Transport>,
}

impl Files {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    /// 上传文件（multipart/form-data）。
    ///
    /// 等价于 `create_with_options(params, RequestOptions::default())`。
    pub async fn create(&self, params: FileCreateParams) -> Result<FileObject> {
        self.create_with_options(params, RequestOptions::default())
            .await
    }

    /// 上传文件（带请求级覆盖项）。
    ///
    /// 文件通过 `multipart/form-data` 格式上传，`purpose` 和 `file` 为必填字段。
    pub async fn create_with_options(
        &self,
        params: FileCreateParams,
        options: RequestOptions,
    ) -> Result<FileObject> {
        let form = build_create_form(&params, &options)?;
        self.transport
            .post_multipart_json("/files", form, options)
            .await
    }

    /// 获取单个文件元信息。
    ///
    /// 等价于 `retrieve_with_options(file_id, RequestOptions::default())`。
    pub async fn retrieve(&self, file_id: impl AsRef<str>) -> Result<FileObject> {
        self.retrieve_with_options(file_id, RequestOptions::default())
            .await
    }

    /// 获取单个文件元信息（带请求级覆盖项）。
    pub async fn retrieve_with_options(
        &self,
        file_id: impl AsRef<str>,
        options: RequestOptions,
    ) -> Result<FileObject> {
        let path = file_path(file_id.as_ref(), None)?;
        self.transport.get_json(&path, options).await
    }

    /// 拉取文件列表（默认参数）。
    pub async fn list(&self) -> Result<CursorPage<FileObject>> {
        self.list_with_params(FileListParams::default()).await
    }

    /// 按显式分页参数拉取文件列表。
    pub async fn list_with_params(&self, params: FileListParams) -> Result<CursorPage<FileObject>> {
        self.list_with_options(params, RequestOptions::default())
            .await
    }

    /// 按参数与请求级覆盖项拉取文件列表。
    pub async fn list_with_options(
        &self,
        params: FileListParams,
        options: RequestOptions,
    ) -> Result<CursorPage<FileObject>> {
        let options = apply_list_params(params, options);
        self.transport.get_json("/files", options).await
    }

    /// 基于当前页计算并拉取下一页。
    ///
    /// 如果当前页没有更多数据（`has_more` 为 `false` 或无可用游标），返回 `Ok(None)`。
    pub async fn list_next_page(
        &self,
        current_page: &CursorPage<FileObject>,
        params: FileListParams,
    ) -> Result<Option<CursorPage<FileObject>>> {
        self.list_next_page_with_options(current_page, params, RequestOptions::default())
            .await
    }

    /// 基于当前页计算并拉取下一页（带请求级覆盖项）。
    pub async fn list_next_page_with_options(
        &self,
        current_page: &CursorPage<FileObject>,
        params: FileListParams,
        options: RequestOptions,
    ) -> Result<Option<CursorPage<FileObject>>> {
        let Some(params) = next_page_params(current_page, params) else {
            return Ok(None);
        };

        let options = pagination_request_options(options);
        self.list_with_options(params, options).await.map(Some)
    }

    /// 返回自动翻页流。
    ///
    /// 流会持续请求下一页直到 `has_more` 为 `false` 或缺失可用游标。
    /// 每次产出单个文件对象，调用方无需手动处理分页逻辑。
    pub fn list_auto_paging(
        &self,
        params: FileListParams,
    ) -> impl Stream<Item = Result<FileObject>> {
        self.list_auto_paging_with_options(params, RequestOptions::default())
    }

    /// 返回自动翻页流（带请求级覆盖项）。
    pub fn list_auto_paging_with_options(
        &self,
        params: FileListParams,
        options: RequestOptions,
    ) -> impl Stream<Item = Result<FileObject>> {
        let files = self.clone();
        let (mut params, options) = pagination_start(params, options);

        try_stream! {
            loop {
                let page = files
                    .list_with_options(params.clone(), options.clone())
                    .await?;
                let next_params = next_page_params(&page, params.clone());

                for item in page.into_items() {
                    yield item;
                }

                let Some(next_params) = next_params else {
                    break;
                };
                params = next_params;
            }
        }
    }

    /// 删除文件。
    ///
    /// 等价于 `delete_with_options(file_id, RequestOptions::default())`。
    pub async fn delete(&self, file_id: impl AsRef<str>) -> Result<FileDeleted> {
        self.delete_with_options(file_id, RequestOptions::default())
            .await
    }

    /// 删除文件（带请求级覆盖项）。
    pub async fn delete_with_options(
        &self,
        file_id: impl AsRef<str>,
        options: RequestOptions,
    ) -> Result<FileDeleted> {
        let path = file_path(file_id.as_ref(), None)?;
        self.transport.delete_json(&path, options).await
    }

    /// 获取文件原始内容（二进制）。
    ///
    /// 等价于 `content_with_options(file_id, RequestOptions::default())`。
    pub async fn content(&self, file_id: impl AsRef<str>) -> Result<Bytes> {
        self.content_with_options(file_id, RequestOptions::default())
            .await
    }

    /// 获取文件原始内容（二进制，带请求级覆盖项）。
    ///
    /// 若调用方未显式设置 `Accept`，默认注入 `application/binary`。
    pub async fn content_with_options(
        &self,
        file_id: impl AsRef<str>,
        mut options: RequestOptions,
    ) -> Result<Bytes> {
        options
            .extra_headers
            .entry("Accept".to_string())
            .or_insert_with(|| "application/binary".to_string());

        let path = file_path(file_id.as_ref(), Some("content"))?;
        self.transport.get_bytes(&path, options).await
    }
}

/// 构建文件上传的 multipart 表单。
///
/// 表单包含：
/// - `purpose`（文本字段，必填）
/// - `file`（文件字段，必填）
/// - `expires_after`（文本字段，可选）
/// - `extra` 和 `options.extra_body` 中的扩展字段
fn build_create_form(
    params: &FileCreateParams,
    options: &RequestOptions,
) -> Result<MultipartFormData> {
    let purpose = purpose_as_str(&params.purpose)?;
    let mut form = MultipartFormData::new()
        .text("purpose", purpose)
        .file(MultipartFile {
            field_name: "file".to_string(),
            file_name: params.file.file_name.clone(),
            bytes: params.file.bytes.clone(),
            mime_type: params.file.mime_type.clone(),
        });

    // 可选的过期策略
    if let Some(expires_after) = &params.expires_after {
        form = form.text("expires_after", serde_json::to_string(expires_after)?);
    }

    // 模型参数中的扩展字段
    for (key, value) in &params.extra {
        form = form.text(key, form_value(value));
    }

    // 请求级 extra_body 中的扩展字段
    if let Some(extra_body) = &options.extra_body {
        let Value::Object(extra_body) = extra_body else {
            return Err(Error::Config(
                "multipart extra_body must be a JSON object".to_string(),
            ));
        };

        for (key, value) in extra_body {
            form = form.text(key, form_value(value));
        }
    }

    Ok(form)
}

/// 将 `FilePurpose` 枚举序列化为 API 要求的字符串值。
fn purpose_as_str(purpose: &FilePurpose) -> Result<String> {
    let value = serde_json::to_value(purpose)?;
    value
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| Error::Config("file purpose must serialize to a string".to_string()))
}

/// 将 JSON 值转换为 multipart 文本字段值。
///
/// 字符串值直接使用，其他类型序列化为 JSON 字符串。
fn form_value(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}

/// 把 `FileListParams` 写入查询参数，且不覆盖调用方已存在的同名参数。
fn apply_list_params(params: FileListParams, mut options: RequestOptions) -> RequestOptions {
    insert_query_if_absent(&mut options, "after", params.after);
    insert_query_if_absent(
        &mut options,
        "limit",
        params.limit.map(|value| value.to_string()),
    );
    insert_query_if_absent(
        &mut options,
        "order",
        params.order.map(|value| match value {
            ListOrder::Asc => "asc".to_string(),
            ListOrder::Desc => "desc".to_string(),
        }),
    );
    insert_query_if_absent(&mut options, "purpose", params.purpose);
    options
}

/// 仅在 key 尚不存在时写入查询参数（避免覆盖调用方显式设置的值）。
fn insert_query_if_absent(options: &mut RequestOptions, key: &str, value: Option<String>) {
    let Some(value) = value else {
        return;
    };

    options.extra_query.entry(key.to_string()).or_insert(value);
}

/// 根据当前页计算下一页参数。
///
/// 返回 `None` 的场景：
/// - 当前页无末尾 item id（`next_after` 为 `None`）；
/// - `has_more` 明确为 `false`。
fn next_page_params(
    current_page: &CursorPage<FileObject>,
    mut params: FileListParams,
) -> Option<FileListParams> {
    let after = current_page.next_after()?;
    if !current_page.has_next_page() {
        return None;
    }

    params.after = Some(after.to_string());
    Some(params)
}

/// 为自动翻页流准备初始参数。
///
/// 如果 `params.after` 未设置但 `options.extra_query` 中有 `after`，
/// 将其移入 `params` 以确保初始游标来源唯一。
fn pagination_start(
    mut params: FileListParams,
    mut options: RequestOptions,
) -> (FileListParams, RequestOptions) {
    if params.after.is_none() {
        params.after = options.extra_query.get("after").cloned();
    }
    // 从 query 参数中移除 after，避免与 params.after 双重来源冲突
    options.extra_query.remove("after");
    (params, options)
}

/// 为后续翻页请求清理 `after`，避免双重来源冲突。
fn pagination_request_options(mut options: RequestOptions) -> RequestOptions {
    options.extra_query.remove("after");
    options
}

/// 构建文件资源路径，并对 `file_id` 做路径安全编码。
///
/// ```text
/// file_path("file_123", None)           → "/files/file_123"
/// file_path("file_123", Some("content")) → "/files/file_123/content"
/// file_path("file/123", None)           → "/files/file%2F123"
/// ```
fn file_path(file_id: &str, suffix: Option<&str>) -> Result<String> {
    if file_id.is_empty() {
        return Err(Error::Config("file_id must not be empty".to_string()));
    }

    let file_id = encode_path_segment(file_id);
    Ok(match suffix {
        Some(suffix) => format!("/files/{file_id}/{suffix}"),
        None => format!("/files/{file_id}"),
    })
}
