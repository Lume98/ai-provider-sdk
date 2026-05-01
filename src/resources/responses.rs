//! Responses 资源封装。统一普通响应与流式响应调用路径。

use std::sync::Arc;

use serde_json::{to_value, Value};

use crate::error::Result;
use crate::request_options::RequestOptions;
use crate::streaming::SseStream;
use crate::transport::Transport;
use crate::types::{Response, ResponseCreateParams};

#[derive(Clone)]
/// Responses 资源入口。
pub struct Responses {
    transport: Arc<Transport>,
}

impl Responses {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    pub async fn create(&self, params: ResponseCreateParams) -> Result<Response> {
        self.create_with_options(params, RequestOptions::default())
            .await
    }

    pub async fn create_with_options(
        &self,
        params: ResponseCreateParams,
        options: RequestOptions,
    ) -> Result<Response> {
        self.transport
            .post_json("/responses", request_body(params, false)?, options)
            .await
    }

    pub async fn create_stream(&self, params: ResponseCreateParams) -> Result<SseStream> {
        self.create_stream_with_options(params, RequestOptions::default())
            .await
    }

    pub async fn create_stream_with_options(
        &self,
        params: ResponseCreateParams,
        options: RequestOptions,
    ) -> Result<SseStream> {
        self.transport
            .post_stream("/responses", request_body(params, true)?, options)
            .await
    }
}

fn request_body(params: ResponseCreateParams, stream: bool) -> Result<Value> {
    let mut body = to_value(params)?;
    if let Value::Object(map) = &mut body {
        map.insert("stream".to_string(), Value::Bool(stream));
    }
    Ok(body)
}
