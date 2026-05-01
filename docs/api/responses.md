# Responses

## 如何使用

### 创建响应

```rust
use openai_rust::{OpenAI, ResponseCreateParams};

# async fn demo() -> Result<(), openai_rust::Error> {
let client = OpenAI::from_env()?;
let response = client
    .responses()
    .create(ResponseCreateParams::new("gpt-4.1-mini").input("hello"))
    .await?;

println!("{}", response.id);
# Ok(())
# }
```

### 流式创建

```rust
use futures_util::StreamExt;
use openai_rust::{OpenAI, ResponseCreateParams};

# async fn demo() -> Result<(), openai_rust::Error> {
let client = OpenAI::from_env()?;
let mut events = client
    .responses()
    .create_stream(ResponseCreateParams::new("gpt-4.1-mini").input("hello"))
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

`ResponseCreateParams`

- `model: String`（必填）目标模型 ID。
- `input: Option<serde_json::Value>`（可选）输入内容，支持字符串、数组、结构化 JSON。
- `instructions: Option<String>`（可选）额外指令。
- `max_output_tokens: Option<u32>`（可选）最大输出 token。
- `metadata: Option<HashMap<String, Value>>`（可选）元信息。
- `temperature: Option<f64>`（可选）采样温度。
- `top_p: Option<f64>`（可选）核采样参数。
- `store: Option<bool>`（可选）是否存储。
- `stream_options: Option<Value>`（可选）流式选项。
- `extra: HashMap<String, Value>`（可选）前向兼容扩展字段，序列化时扁平展开。

便捷构造：

- `ResponseCreateParams::new(model)`：设置 `model`。
- `.input(value)`：设置 `input`。

## 响应结构（全量）

`Response`

- `id: String`：响应 ID。
- `extra: HashMap<String, Value>`：服务端新增字段容器。

`ResponseStreamEvent`（SDK 解析的流式事件 payload）

- `type: Option<String>`：事件类型。
- `extra: HashMap<String, Value>`：事件扩展字段。

SSE 外层事件结构见 [/api/streaming](/api/streaming)。

## 兼容性说明

- `extra` 仅用于前向兼容，不应当作为稳定契约字段。
- 文档只覆盖当前仓库已实现能力。
