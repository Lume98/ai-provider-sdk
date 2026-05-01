use std::sync::Arc;

use serde_json::{to_value, Value};

use crate::error::Result;
use crate::request_options::RequestOptions;
use crate::streaming::SseStream;
use crate::transport::Transport;
use crate::types::{ChatCompletion, ChatCompletionCreateParams};

#[derive(Clone)]
pub struct Chat {
    transport: Arc<Transport>,
}

impl Chat {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    pub fn completions(&self) -> ChatCompletions {
        ChatCompletions {
            transport: self.transport.clone(),
        }
    }
}

#[derive(Clone)]
pub struct ChatCompletions {
    transport: Arc<Transport>,
}

impl ChatCompletions {
    pub async fn create(&self, params: ChatCompletionCreateParams) -> Result<ChatCompletion> {
        self.create_with_options(params, RequestOptions::default())
            .await
    }

    pub async fn create_with_options(
        &self,
        params: ChatCompletionCreateParams,
        options: RequestOptions,
    ) -> Result<ChatCompletion> {
        self.transport
            .post_json("/chat/completions", request_body(params, false)?, options)
            .await
    }

    pub async fn create_stream(&self, params: ChatCompletionCreateParams) -> Result<SseStream> {
        self.create_stream_with_options(params, RequestOptions::default())
            .await
    }

    pub async fn create_stream_with_options(
        &self,
        params: ChatCompletionCreateParams,
        options: RequestOptions,
    ) -> Result<SseStream> {
        self.transport
            .post_stream("/chat/completions", request_body(params, true)?, options)
            .await
    }
}

fn request_body(params: ChatCompletionCreateParams, stream: bool) -> Result<Value> {
    let mut body = to_value(params)?;
    if let Value::Object(map) = &mut body {
        map.insert("stream".to_string(), Value::Bool(stream));
    }
    Ok(body)
}
