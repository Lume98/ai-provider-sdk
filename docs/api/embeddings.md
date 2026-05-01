# Embeddings

## 已实现方法

- `create`
- `create_with_options`

## 行为说明

`EmbeddingCreateParams.encoding_format` 未显式设置时，SDK 默认补成 `float`。

## 示例

```rust
use openai_rust::{EmbeddingCreateParams, OpenAI};

# async fn demo() -> Result<(), openai_rust::Error> {
let client = OpenAI::from_env()?;
let _resp = client
    .embeddings()
    .create(EmbeddingCreateParams::new("text-embedding-3-small", "hello"))
    .await?;
# Ok(())
# }
```
