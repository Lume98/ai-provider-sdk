use std::sync::Arc;

use crate::error::{Error, Result};
use crate::path::encode_path_segment;
use crate::request_options::RequestOptions;
use crate::transport::Transport;
use crate::types::{Model, ModelList};

#[derive(Clone)]
pub struct Models {
    transport: Arc<Transport>,
}

impl Models {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    pub async fn list(&self) -> Result<ModelList> {
        self.list_with_options(RequestOptions::default()).await
    }

    pub async fn list_with_options(&self, options: RequestOptions) -> Result<ModelList> {
        self.transport.get_json("/models", options).await
    }

    pub async fn retrieve(&self, model: impl AsRef<str>) -> Result<Model> {
        self.retrieve_with_options(model, RequestOptions::default())
            .await
    }

    pub async fn retrieve_with_options(
        &self,
        model: impl AsRef<str>,
        options: RequestOptions,
    ) -> Result<Model> {
        let model = model.as_ref();
        if model.is_empty() {
            return Err(Error::Config("model must not be empty".to_string()));
        }

        let model = encode_path_segment(model);
        self.transport
            .get_json(&format!("/models/{model}"), options)
            .await
    }
}
