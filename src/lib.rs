//! # ai-provider-sdk
//!
//! OpenAI 兼容 API 的 Rust 异步客户端 SDK。
//!
//! ## 架构概览
//!
//! ```text
//! lib.rs          ← 模块装配与公共符号重导出（本文件）
//! ├── client.rs   ← 客户端构建（`OpenAI`、`ClientOptions`）
//! ├── transport.rs ← HTTP 传输层（重试、URL 组装、请求发送）
//! ├── streaming.rs ← SSE 流式解码
//! ├── resources/  ← 各 API 资源入口（Chat、Embeddings、Files 等）
//! ├── types/      ← 请求参数与响应体的数据模型
//! ├── error.rs    ← 统一错误类型
//! ├── pagination.rs ← 游标分页抽象
//! ├── request_options.rs ← 单次请求覆盖选项
//! ├── path.rs     ← URL 路径片段编码
//! └── workload.rs ← Workload Identity 联合身份认证
//! ```
//!
//! ## 快速开始
//!
//! ```no_run
//! use ai_provider_sdk::{OpenAI, ChatCompletionCreateParams, ChatMessage};
//!
//! #[tokio::main]
//! async fn main() -> ai_provider_sdk::Result<()> {
//!     // 从环境变量 OPENAI_API_KEY 读取密钥
//!     let client = OpenAI::from_env()?;
//!
//!     // 发起一次 Chat Completion 请求
//!     let completion = client
//!         .chat()
//!         .completions()
//!         .create(ChatCompletionCreateParams::new(
//!             "gpt-4.1-mini",
//!             vec![ChatMessage::user("Hello!")],
//!         ))
//!         .await?;
//!
//!     println!("{:?}", completion);
//!     Ok(())
//! }
//! ```

mod client;
mod error;
mod pagination;
mod path;
mod request_options;
pub mod resources;
mod streaming;
mod transport;
mod workload;
pub mod types;

/// SDK 主入口客户端，提供所有 API 资源的访问方法。
///
/// 通过 `OpenAI::new()`、`OpenAI::from_env()` 或 `OpenAI::with_options()` 构造。
pub use client::{ClientOptions, OpenAI};

/// Workload Identity 联合身份认证相关类型。
///
/// 适用于 GCP / AWS / Azure 等云环境的 Workload Identity 场景，
/// 无需静态 API Key，通过服务账号身份自动换取访问令牌。
pub use workload::{SubjectTokenProvider, SubjectTokenType, WorkloadIdentity};

/// 统一错误类型与别名。
///
/// - `Error` 涵盖配置错误、网络错误、超时、API 状态码错误等。
/// - `Result<T>` 是 `std::result::Result<T, Error>` 的别名。
pub use error::{ApiErrorBody, Error, Result};

/// 游标分页泛型容器。
///
/// - `CursorPage<T>` 封装了 `data` / `has_more` 分页协议。
/// - `CursorPageItem` trait 约束了参与翻页的元素需提供 `id()`。
pub use pagination::{CursorPage, CursorPageItem};

/// 单次请求覆盖选项，可追加请求头、查询参数、JSON body 扩展字段与超时。
pub use request_options::RequestOptions;

/// SSE 流式事件类型与流包装器。
///
/// - `ServerSentEvent` 表示一个完整的 SSE 事件。
/// - `SseStream` 将 HTTP 响应体按 SSE 协议解码为异步事件流。
pub use streaming::{ServerSentEvent, SseStream};

/// 所有 API 请求参数与响应体的数据模型（chat、embeddings、files 等）。
pub use types::*;
