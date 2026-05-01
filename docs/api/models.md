# Models

## 如何使用

```rust
use ai_provider_sdk::OpenAI;

# async fn demo() -> Result<(), ai_provider_sdk::Error> {
let client = OpenAI::from_env()?;

let models = client.models().list().await?;
println!("count={}", models.data.len());

let model = client.models().retrieve("gpt-4.1-mini").await?;
println!("id={}", model.id);
# Ok(())
# }
```

## 已实现方法

- `list()`
- `list_with_options(options)`
- `retrieve(model)`
- `retrieve_with_options(model, options)`

## 入参结构（全量）

- `retrieve(model)` 的 `model: impl AsRef<str>`（必填）为模型 ID，接受 `&str`、`String` 等类型。
- `list()` 无业务参数。
- `*_with_options` 可传 `RequestOptions` 覆盖 header/query/timeout/extra_body。

## 响应结构（全量）

`Model`

- `id: String`
- `object: Option<String>`
- `created: Option<u64>`
- `owned_by: Option<String>`
- `extra: HashMap<String, Value>`

`ModelList`

- `object: Option<String>`
- `data: Vec<Model>`
- `extra: HashMap<String, Value>`

## 兼容性说明

- `extra` 为前向兼容容器，不保证稳定结构。
- 文档只覆盖当前仓库已实现能力。
