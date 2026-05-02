# Chat Completions

## 如何使用

### 创建补全

```rust
use ai_provider_sdk::{ChatCompletionCreateParams, ChatMessage, OpenAI};

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
```

### 流式补全

```rust
use futures_util::StreamExt;
use ai_provider_sdk::{ChatCompletionCreateParams, ChatMessage, OpenAI};

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
```

## 已实现方法

- `create(params)`
- `create_with_options(params, options)`
- `create_stream(params)`
- `create_stream_with_options(params, options)`
- `retrieve(completion_id)`
- `retrieve_with_options(completion_id, options)`
- `update(completion_id, params)`
- `update_with_options(completion_id, params, options)`
- `list()`
- `list_with_params(params)`
- `list_with_options(params, options)`
- `delete(completion_id)`
- `delete_with_options(completion_id, options)`
- `messages().list(completion_id)`
- `messages().list_with_params(completion_id, params)`
- `messages().list_with_options(completion_id, params, options)`

## 入参结构（强类型字段）

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

`ChatCompletionListParams`

- `after: Option<String>`（可选）分页游标。
- `limit: Option<u32>`（可选）单页数量上限。
- `metadata: Option<HashMap<String, String>>`（可选）metadata 过滤。
- `model: Option<String>`（可选）模型过滤。
- `order: Option<ChatListOrder>`（可选）排序方向。

`ChatCompletionUpdateParams`

- `metadata: HashMap<String, String>`：stored completion metadata。

`ChatCompletionMessageListParams`

- `after: Option<String>`（可选）分页游标。
- `limit: Option<u32>`（可选）单页数量上限。
- `order: Option<ChatListOrder>`（可选）排序方向。

`ChatListOrder`

- `Asc`
- `Desc`

## 响应结构（强类型字段）

`ChatCompletion`

- `id: String`：补全 ID。
- `extra: HashMap<String, Value>`：服务端新增字段容器。

`ChatCompletionChunk`（流式 payload）

- `id: String`：chunk 对应 ID。
- `extra: HashMap<String, Value>`：chunk 扩展字段。

`ChatCompletionDeleted`

- `id: String`：被删除的补全 ID。
- `deleted: bool`：是否删除成功。
- `object: Option<String>`：对象类型标识。
- `extra: HashMap<String, Value>`：服务端新增字段容器。

`ChatCompletionStoreMessage`

- `id: String`：消息 ID。
- `role: Option<ChatRole>`：消息角色。
- `content: Option<Value>`：消息内容。
- `object: Option<String>`：对象类型标识。
- `created_at: Option<u64>`：创建时间。
- `extra: HashMap<String, Value>`：服务端新增字段容器。

SSE 外层事件结构见 [/api/streaming](/api/streaming)。

## 兼容性说明

- `extra` 为前向兼容容器，不保证稳定结构。
- 文档只覆盖当前仓库强类型暴露的字段，不代表 OpenAI API 全量参数。
- `retrieve` / `update` / `list` / `delete` / `messages().list` 只适用于服务端已存储的 Chat Completion；通常需要创建时设置 `store=true`。
