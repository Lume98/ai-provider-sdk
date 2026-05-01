//! SDK 对外导出入口。
//! 该文件只负责模块装配与公共符号重导出，不承载业务逻辑。

mod client;
mod error;
mod pagination;
mod path;
mod request_options;
pub mod resources;
mod streaming;
mod transport;
pub mod types;

pub use client::{ClientOptions, OpenAI};
pub use error::{ApiErrorBody, Error, Result};
pub use pagination::{CursorPage, CursorPageItem};
pub use request_options::RequestOptions;
pub use streaming::{ServerSentEvent, SseStream};
pub use types::*;
