# Moderations

## 已实现方法

- `create`
- `create_with_options`

## 示例

```rust
use openai_rust::{ModerationCreateParams, OpenAI};

# async fn demo() -> Result<(), openai_rust::Error> {
let client = OpenAI::from_env()?;
let _resp = client
    .moderations()
    .create(ModerationCreateParams::new("hello").model("omni-moderation-latest"))
    .await?;
# Ok(())
# }
```
