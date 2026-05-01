# Models

## 已实现方法

- `list`
- `list_with_options`
- `retrieve`
- `retrieve_with_options`

## 示例

```rust
use openai_rust::OpenAI;

# async fn demo() -> Result<(), openai_rust::Error> {
let client = OpenAI::from_env()?;
let models = client.models().list().await?;
println!("{}", models.data.len());
# Ok(())
# }
```
