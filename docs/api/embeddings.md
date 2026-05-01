# Embeddings

## 如何使用

```rust
use ai_provider_sdk::{EmbeddingCreateParams, OpenAI};

let client = OpenAI::from_env()?;

let resp = client
    .embeddings()
    .create(EmbeddingCreateParams::new("text-embedding-3-small", "hello"))
    .await?;

println!("{}", resp.data.len());
```

## 已实现方法

- `create(params)`
- `create_with_options(params, options)`

## 入参结构（全量）

`EmbeddingCreateParams`

- `model: String`（必填）目标模型 ID。
- `input: EmbeddingInput`（必填）向量输入。
- `dimensions: Option<u32>`（可选）向量维度。
- `encoding_format: Option<EncodingFormat>`（可选）返回编码格式。
- `user: Option<String>`（可选）终端用户标识。
- `extra: HashMap<String, Value>`（可选）扩展字段。

`EmbeddingInput`（联合类型）

- `Text(String)`：单条文本。
- `Texts(Vec<String>)`：批量文本。
- `Tokens(Vec<u32>)`：单条 token 序列。
- `TokenBatches(Vec<Vec<u32>>)`：批量 token 序列。

`EncodingFormat` 枚举值：

- `Float`
- `Base64`

默认行为：

- `encoding_format` 未设置时，SDK 默认发送 `float`。

## 响应结构（全量）

`CreateEmbeddingResponse`

- `object: Option<String>`
- `data: Vec<Embedding>`
- `model: Option<String>`
- `usage: Option<EmbeddingUsage>`
- `extra: HashMap<String, Value>`

`Embedding`

- `object: Option<String>`
- `index: u32`
- `embedding: EmbeddingVector`
- `extra: HashMap<String, Value>`

`EmbeddingVector`（联合类型）

- `Float(Vec<f64>)`
- `Base64(String)`

`EmbeddingUsage`

- `prompt_tokens: u32`
- `total_tokens: u32`

## 兼容性说明

- `extra` 为前向兼容容器，不保证稳定结构。
- 文档只覆盖当前仓库已实现能力。
