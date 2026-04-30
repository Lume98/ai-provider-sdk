use bytes::Bytes;
use reqwest::{Method, multipart};
use serde::{Deserialize, Serialize};

use super::{
    DeletedObject, FilesResource, JsonObject, ListPage, ListParams, RequestOptions,
    UploadsResource, core::path_segment, generic::GenericCreateParams, generic::GenericObject,
};
use crate::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileObject {
    pub id: String,
    pub object: String,
    #[serde(default)]
    pub bytes: Option<u64>,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub purpose: Option<String>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct FileListParams {
    #[serde(flatten)]
    pub page: ListParams,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
}

pub struct FileCreateParams {
    pub file_name: String,
    pub file_bytes: Bytes,
    pub purpose: String,
}

impl FileCreateParams {
    pub fn from_bytes(
        file_name: impl Into<String>,
        file_bytes: impl Into<Bytes>,
        purpose: impl Into<String>,
    ) -> Self {
        Self {
            file_name: file_name.into(),
            file_bytes: file_bytes.into(),
            purpose: purpose.into(),
        }
    }
}

impl FilesResource {
    pub async fn create(&self, params: FileCreateParams) -> Result<FileObject, Error> {
        let part = multipart::Part::bytes(params.file_bytes.to_vec()).file_name(params.file_name);
        let form = multipart::Form::new()
            .part("file", part)
            .text("purpose", params.purpose);
        self.core
            .multipart("/files", form, RequestOptions::default())
            .await
    }

    pub async fn list(&self, params: &FileListParams) -> Result<ListPage<FileObject>, Error> {
        self.core
            .json_value(
                Method::GET,
                "/files",
                Some(params),
                Option::<&()>::None,
                RequestOptions::default(),
            )
            .await
    }

    pub async fn retrieve(&self, file_id: &str) -> Result<FileObject, Error> {
        self.core
            .json_value(
                Method::GET,
                &format!("/files/{}", path_segment(file_id)),
                Option::<&()>::None,
                Option::<&()>::None,
                RequestOptions::default(),
            )
            .await
    }

    pub async fn delete(&self, file_id: &str) -> Result<DeletedObject, Error> {
        self.core
            .json_value(
                Method::DELETE,
                &format!("/files/{}", path_segment(file_id)),
                Option::<&()>::None,
                Option::<&()>::None,
                RequestOptions::default(),
            )
            .await
    }

    pub async fn content(&self, file_id: &str) -> Result<Bytes, Error> {
        self.core
            .bytes(
                Method::GET,
                &format!("/files/{}/content", path_segment(file_id)),
                Option::<&()>::None,
                RequestOptions::default(),
            )
            .await
    }
}

impl UploadsResource {
    pub async fn create(&self, params: &GenericCreateParams) -> Result<GenericObject, Error> {
        self.core
            .json_value(
                Method::POST,
                "/uploads",
                Option::<&()>::None,
                Some(params),
                RequestOptions::default(),
            )
            .await
    }

    pub async fn add_part(
        &self,
        upload_id: &str,
        params: FileCreateParams,
    ) -> Result<GenericObject, Error> {
        let part = multipart::Part::bytes(params.file_bytes.to_vec()).file_name(params.file_name);
        let form = multipart::Form::new().part("data", part);
        self.core
            .multipart(
                &format!("/uploads/{}/parts", path_segment(upload_id)),
                form,
                RequestOptions::default(),
            )
            .await
    }

    pub async fn complete(
        &self,
        upload_id: &str,
        params: &GenericCreateParams,
    ) -> Result<GenericObject, Error> {
        self.core
            .json_value(
                Method::POST,
                &format!("/uploads/{}/complete", path_segment(upload_id)),
                Option::<&()>::None,
                Some(params),
                RequestOptions::default(),
            )
            .await
    }

    pub async fn cancel(&self, upload_id: &str) -> Result<GenericObject, Error> {
        self.core
            .json_value(
                Method::POST,
                &format!("/uploads/{}/cancel", path_segment(upload_id)),
                Option::<&()>::None,
                Option::<&()>::None,
                RequestOptions::default(),
            )
            .await
    }
}
