use std::sync::Arc;

use serde_json::{to_value, Value};

use crate::error::Result;
use crate::request_options::RequestOptions;
use crate::transport::Transport;
use crate::types::{CreateEmbeddingResponse, EmbeddingCreateParams, EncodingFormat};

#[derive(Clone)]
pub struct Embeddings {
    transport: Arc<Transport>,
}

impl Embeddings {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    pub async fn create(&self, params: EmbeddingCreateParams) -> Result<CreateEmbeddingResponse> {
        self.create_with_options(params, RequestOptions::default())
            .await
    }

    pub async fn create_with_options(
        &self,
        params: EmbeddingCreateParams,
        options: RequestOptions,
    ) -> Result<CreateEmbeddingResponse> {
        self.transport
            .post_json("/embeddings", request_body(params)?, options)
            .await
    }
}

fn request_body(mut params: EmbeddingCreateParams) -> Result<Value> {
    if params.encoding_format.is_none() {
        params.encoding_format = Some(EncodingFormat::Float);
    }

    to_value(params).map_err(Into::into)
}
