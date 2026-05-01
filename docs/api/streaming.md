# Streaming

`create_stream(...)` 返回 `SseStream`，调用 `.events()` 得到 `Stream<Item = Result<ServerSentEvent>>`。

事件处理规则：

- 收到 `data: [DONE]` 结束流。
- 事件 `data` 若包含 `{"error": ...}`，转换为 `Error::Stream`。
- 支持标准 SSE 字段：`event`、`data`、`id`、`retry`。其中 `id` 和 `retry` 由 `ServerSentEvent` 结构体解析保存，但当前 SDK 未对其做特殊处理（`retry` 不影响客户端重连间隔）。

支持流式的资源：

- `responses.create_stream(...)`
- `chat.completions.create_stream(...)`
