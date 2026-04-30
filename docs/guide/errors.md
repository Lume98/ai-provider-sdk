# 错误处理

SDK 统一使用 `vendor_sdk::Error`：

- `Http`：请求层错误
- `MissingEnv`：缺少必要环境变量
- `Api`：OpenAI API 返回错误（含 `status`、`code`、`message`）
- `Serde`：序列化/反序列化错误
- `StreamProtocol`：SSE 事件协议错误
- `Multipart`：文件与 multipart 处理错误
- `WebhookVerification`：webhook 签名校验失败
- `Unsupported`：暂不支持的操作

典型做法：将 `Error::Api` 记录完整字段，其他错误记录 `to_string()` 并附带请求上下文。
