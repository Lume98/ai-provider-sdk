//! 测试辅助场景模块。
//!
//! 按 SDK 功能领域组织，每个子模块对应一类横切关注点的测试：
//! - `errors`：错误模型与错误处理行为
//! - `request_options`：单次请求覆盖选项行为
//! - `retries`：重试策略与幂等键行为
//! - `streaming`：SSE 流式解码行为

mod errors;
mod request_options;
mod retries;
mod streaming;
