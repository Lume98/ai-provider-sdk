# Examples

运行示例前先配置环境变量：

```bash
export OPENAI_API_KEY="sk-..."
export OPENAI_CHAT_MODEL="gpt-4.1-mini"
```

## Chat Completions

`chat.rs` 对齐 `openai-python/examples/demo.py` 的三个使用面：

- 普通 Chat Completion 请求。
- 流式 Chat Completion 请求。
- 请求级配置示例，对应本 SDK 的 `RequestOptions`。

```bash
cargo run --example chat
```

当前 SDK 的 `ChatCompletion` 只强类型暴露 `id`，`choices`、`usage` 等字段会进入 `extra`。示例通过 `extra["choices"][0]["message"]["content"]` 读取文本，这是为了保留 OpenAI API 新字段的前向兼容性。

设计边界：`openai-python` 的 `with_raw_response` 能在成功响应里读取 `request_id`；本 SDK 当前只在错误响应 `Error::ApiStatus` 中暴露 `request_id`，尚未提供成功响应的 raw response 包装。

## Models

`models.rs` 演示 Models 资源的三个使用面：

- 列出所有可用模型。
- 查询单个模型详情。
- 请求级配置示例，对应本 SDK 的 `RequestOptions`。

```bash
cargo run --example models
```
