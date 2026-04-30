# Files & Uploads

## 上传文件

```rust
use bytes::Bytes;
use vendor_sdk::{FileCreateParams, OpenAIClient};

# async fn demo() -> Result<(), vendor_sdk::Error> {
let client = OpenAIClient::from_env()?;
let file = client
    .files
    .create(FileCreateParams::from_bytes(
        "data.jsonl",
        Bytes::from_static(b"{\"messages\":[]}\n"),
        "fine-tune",
    ))
    .await?;
# Ok(())
# }
```

## Files 方法

- `create(FileCreateParams)`
- `list(&FileListParams)`
- `retrieve(&str)`
- `delete(&str)`
- `content(&str)`

## Uploads 方法

- `create(&GenericCreateParams)`
- `add_part(&str, FileCreateParams)`
- `complete(&str, &GenericCreateParams)`
- `cancel(&str)`
