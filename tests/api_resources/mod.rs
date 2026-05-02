//! API 资源集成测试模块。
//!
//! 每个子模块对应一个 API 资源，使用 `httpmock` 模拟 HTTP 服务端，
//! 验证 SDK 的请求构造、响应解析、流式解码和错误处理行为。

mod chat;
mod embeddings;
mod files;
mod models;
mod moderations;
mod responses;
