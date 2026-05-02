//! 单次请求覆盖选项。
//!
//! 用于在全局默认配置之外对某一次请求做局部覆写。
//! 优先级：调用级 [`RequestOptions`] > 客户端默认配置。
//!
//! ## 使用方式
//!
//! ```no_run
//! use std::time::Duration;
//! use ai_provider_sdk::{OpenAI, RequestOptions};
//! use serde_json::json;
//!
//! # async fn example(client: OpenAI) -> ai_provider_sdk::Result<()> {
//! let options = RequestOptions::new()
//!     .header("x-trace-id", "abc-123")             // 追加追踪头
//!     .query("api-version", "2024-01-01")           // 追加查询参数
//!     .extra_body(json!({"metadata": {"env": "dev"}})) // 追加请求体字段
//!     .timeout(Duration::from_secs(30));            // 覆盖超时
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::time::Duration;

use serde_json::Value;

#[derive(Debug, Clone, Default)]
/// 单次请求覆盖项。
///
/// 所有字段均为可选追加/覆盖语义，不会清除客户端级别的默认配置。
pub struct RequestOptions {
    /// 追加/覆盖本次请求的 HTTP 头。
    ///
    /// 同名键会覆盖客户端默认头。典型用途：注入追踪头、灰度头、临时鉴权头。
    pub extra_headers: HashMap<String, String>,

    /// 追加本次请求的查询参数。
    ///
    /// 与客户端默认查询参数合并；同名键以本字段写入顺序参与拼接。
    pub extra_query: HashMap<String, String>,

    /// 追加到 JSON 请求体的额外字段。
    ///
    /// - 当原请求体与该值均为 JSON 对象时，按键合并，冲突键由 `extra_body` 覆盖。
    /// - 非对象场景下，`extra_body` 将直接替换原请求体。
    pub extra_body: Option<Value>,

    /// 覆盖本次请求超时（含连接后请求阶段）。
    pub timeout: Option<Duration>,
}

impl RequestOptions {
    /// 创建空覆盖项，等价于 `Default::default()`。
    pub fn new() -> Self {
        Self::default()
    }

    /// 追加一个请求头（builder 模式）。
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_headers.insert(key.into(), value.into());
        self
    }

    /// 追加一个查询参数（builder 模式）。
    pub fn query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_query.insert(key.into(), value.into());
        self
    }

    /// 追加 JSON 请求体扩展字段（builder 模式）。
    pub fn extra_body(mut self, value: Value) -> Self {
        self.extra_body = Some(value);
        self
    }

    /// 设置本次请求超时覆盖值（builder 模式）。
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}
