# Chat Completions

## 如何使用

### 创建补全

```rust
use openai_rust::{ChatCompletionCreateParams, ChatMessage, OpenAI};

# async fn demo() -> Result<(), openai_rust::Error> {
let client = OpenAI::from_env()?;
let completion = client
    .chat()
    .completions()
    .create(ChatCompletionCreateParams::new(
        "gpt-4.1-mini",
        vec![ChatMessage::user("hello")],
    ))
    .await?;

println!("{}", completion.id);
# Ok(())
# }
```

### 流式补全

```rust
use futures_util::StreamExt;
use openai_rust::{ChatCompletionCreateParams, ChatMessage, OpenAI};

# async fn demo() -> Result<(), openai_rust::Error> {
let client = OpenAI::from_env()?;
let mut events = client
    .chat()
    .completions()
    .create_stream(ChatCompletionCreateParams::new(
        "gpt-4.1-mini",
        vec![ChatMessage::user("hello")],
    ))
    .await?
    .events();

while let Some(event) = events.next().await {
    let event = event?;
    println!("event={:?}, data={}", event.event, event.data);
}
# Ok(())
# }
```

## 已实现方法

- `create(params)`
- `create_with_options(params, options)`
- `create_stream(params)`
- `create_stream_with_options(params, options)`

## 入参结构（全量）

`ChatCompletionCreateParams`

- `model: String`（必填）目标模型 ID。
- `messages: Vec<ChatMessage>`（必填）消息列表。
- `temperature: Option<f64>`（可选）采样温度。
- `top_p: Option<f64>`（可选）核采样参数。
- `max_completion_tokens: Option<u32>`（可选）最大完成 token。
- `max_tokens: Option<u32>`（可选）兼容字段。
- `stream_options: Option<Value>`（可选）流式选项。
- `store: Option<bool>`（可选）是否存储。
- `extra: HashMap<String, Value>`（可选）前向兼容扩展字段。

`ChatMessage`

- `role: ChatRole`（必填）角色。
- `content: Value`（必填）消息内容，支持文本与结构化 JSON。
- `extra: HashMap<String, Value>`（可选）扩展字段。

`ChatRole` 枚举值：

- `System`
- `Developer`
- `User`
- `Assistant`
- `Tool`

便捷构造：

- `ChatMessage::user(content)`
- `ChatMessage::developer(content)`
- `ChatCompletionCreateParams::new(model, messages)`

## 响应结构（全量）

`ChatCompletion`

- `id: String`：补全 ID。
- `extra: HashMap<String, Value>`：服务端新增字段容器。

`ChatCompletionChunk`（流式 payload）

- `id: String`：chunk 对应 ID。
- `extra: HashMap<String, Value>`：chunk 扩展字段。

SSE 外层事件结构见 [/api/streaming](/api/streaming)。

## 兼容性说明

- `extra` 为前向兼容容器，不保证稳定结构。
- 文档只覆盖当前仓库已实现能力。
