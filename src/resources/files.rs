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
pub struct Files {
    transport: Arc<Transport>,
}

impl Files {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    pub async fn create(&self, params: FileCreateParams) -> Result<FileObject> {
        self.create_with_options(params, RequestOptions::default())
            .await
    }

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

    pub async fn retrieve(&self, file_id: impl AsRef<str>) -> Result<FileObject> {
        self.retrieve_with_options(file_id, RequestOptions::default())
            .await
    }

    pub async fn retrieve_with_options(
        &self,
        file_id: impl AsRef<str>,
        options: RequestOptions,
    ) -> Result<FileObject> {
        let path = file_path(file_id.as_ref(), None)?;
        self.transport.get_json(&path, options).await
    }

    pub async fn list(&self) -> Result<CursorPage<FileObject>> {
        self.list_with_params(FileListParams::default()).await
    }

    pub async fn list_with_params(&self, params: FileListParams) -> Result<CursorPage<FileObject>> {
        self.list_with_options(params, RequestOptions::default())
            .await
    }

    pub async fn list_with_options(
        &self,
        params: FileListParams,
        options: RequestOptions,
    ) -> Result<CursorPage<FileObject>> {
        let options = apply_list_params(params, options);
        self.transport.get_json("/files", options).await
    }

    pub async fn list_next_page(
        &self,
        current_page: &CursorPage<FileObject>,
        params: FileListParams,
    ) -> Result<Option<CursorPage<FileObject>>> {
        self.list_next_page_with_options(current_page, params, RequestOptions::default())
            .await
    }

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

    pub fn list_auto_paging(
        &self,
        params: FileListParams,
    ) -> impl Stream<Item = Result<FileObject>> {
        self.list_auto_paging_with_options(params, RequestOptions::default())
    }

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

    pub async fn delete(&self, file_id: impl AsRef<str>) -> Result<FileDeleted> {
        self.delete_with_options(file_id, RequestOptions::default())
            .await
    }

    pub async fn delete_with_options(
        &self,
        file_id: impl AsRef<str>,
        options: RequestOptions,
    ) -> Result<FileDeleted> {
        let path = file_path(file_id.as_ref(), None)?;
        self.transport.delete_json(&path, options).await
    }

    pub async fn content(&self, file_id: impl AsRef<str>) -> Result<Bytes> {
        self.content_with_options(file_id, RequestOptions::default())
            .await
    }

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

    if let Some(expires_after) = &params.expires_after {
        form = form.text("expires_after", serde_json::to_string(expires_after)?);
    }

    for (key, value) in &params.extra {
        form = form.text(key, form_value(value));
    }

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

fn purpose_as_str(purpose: &FilePurpose) -> Result<String> {
    let value = serde_json::to_value(purpose)?;
    value
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| Error::Config("file purpose must serialize to a string".to_string()))
}

fn form_value(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}

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

fn insert_query_if_absent(options: &mut RequestOptions, key: &str, value: Option<String>) {
    let Some(value) = value else {
        return;
    };

    options.extra_query.entry(key.to_string()).or_insert(value);
}

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

fn pagination_start(
    mut params: FileListParams,
    mut options: RequestOptions,
) -> (FileListParams, RequestOptions) {
    if params.after.is_none() {
        params.after = options.extra_query.get("after").cloned();
    }
    options.extra_query.remove("after");
    (params, options)
}

fn pagination_request_options(mut options: RequestOptions) -> RequestOptions {
    options.extra_query.remove("after");
    options
}

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
