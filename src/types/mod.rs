//! API 类型聚合导出。
//!
//! 集中管理所有 API 的请求参数与响应体数据模型。
//! 每个子模块对应一个 API 领域，通过 `pub use *` 统一导出，
//! 外部可直接使用 `ai_provider_sdk::ChatMessage` 等类型。
//!
//! ## 设计原则
//!
//! - 所有结构体使用 `#[serde(flatten)] extra` 字段捕获未知字段，
//!   确保服务端增量字段不会导致反序列化失败。
//! - 枚举类型使用 `#[serde(untagged)]` 或 `#[serde(rename_all)]` 与 API wire 格式对齐。
//! - 可选字段统一使用 `Option<T>` + `#[serde(default)]`。

pub mod chat;
pub mod embeddings;
pub mod files;
pub mod models;
pub mod moderations;
pub mod responses;

pub use chat::*;
pub use embeddings::*;
pub use files::*;
pub use models::*;
pub use moderations::*;
pub use responses::*;
