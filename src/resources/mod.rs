//! API 资源分组导出。
//!
//! 每个子模块对应一个 OpenAI API 领域，提供该领域的高层调用接口。
//! 所有资源类型均通过 `pub use` 重新导出，便于外部直接使用。
//!
//! ## 模块结构
//!
//! | 模块            | API 路径              | 功能描述              |
//! |----------------|----------------------|---------------------|
//! | `chat`         | `/chat/completions`  | 聊天补全（同步/流式）     |
//! | `embeddings`   | `/embeddings`        | 文本向量化             |
//! | `files`        | `/files`             | 文件上传/下载/管理       |
//! | `models`       | `/models`            | 模型列表与详情查询        |
//! | `moderations`  | `/moderations`       | 内容审核               |
//! | `responses`    | `/responses`         | Responses API（新一代） |

mod chat;
mod embeddings;
mod files;
mod models;
mod moderations;
mod responses;

pub use chat::{Chat, ChatCompletionMessages, ChatCompletions};
pub use embeddings::Embeddings;
pub use files::Files;
pub use models::Models;
pub use moderations::Moderations;
pub use responses::Responses;
