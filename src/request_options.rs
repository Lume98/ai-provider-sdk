use std::collections::HashMap;
use std::time::Duration;

use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct RequestOptions {
    pub extra_headers: HashMap<String, String>,
    pub extra_query: HashMap<String, String>,
    pub extra_body: Option<Value>,
    pub timeout: Option<Duration>,
}

impl RequestOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_headers.insert(key.into(), value.into());
        self
    }

    pub fn query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_query.insert(key.into(), value.into());
        self
    }

    pub fn extra_body(mut self, value: Value) -> Self {
        self.extra_body = Some(value);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}
