use std::sync::Arc;

use serde_json::{to_value, Value};

use crate::error::Result;
use crate::request_options::RequestOptions;
use crate::transport::Transport;
use crate::types::{CreateModerationResponse, ModerationCreateParams};

#[derive(Clone)]
pub struct Moderations {
    transport: Arc<Transport>,
}

impl Moderations {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    pub async fn create(&self, params: ModerationCreateParams) -> Result<CreateModerationResponse> {
        self.create_with_options(params, RequestOptions::default())
            .await
    }

    pub async fn create_with_options(
        &self,
        params: ModerationCreateParams,
        options: RequestOptions,
    ) -> Result<CreateModerationResponse> {
        self.transport
            .post_json("/moderations", request_body(params)?, options)
            .await
    }
}

fn request_body(params: ModerationCreateParams) -> Result<Value> {
    to_value(params).map_err(Into::into)
}
