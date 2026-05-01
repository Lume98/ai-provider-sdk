# Files

## 上传

```rust
use bytes::Bytes;
use openai_rust::{FileCreateParams, FilePurpose, OpenAI, UploadFile};

# async fn demo() -> Result<(), openai_rust::Error> {
let client = OpenAI::from_env()?;
let params = FileCreateParams::new(
    UploadFile::from_bytes("train.jsonl", Bytes::from_static(b"{\"messages\":[]}\n")),
    FilePurpose::FineTune,
);
let file = client.files().create(params).await?;
println!("{}", file.id);
# Ok(())
# }
```

## 分页

```rust
use futures_util::StreamExt;
use openai_rust::{FileListParams, OpenAI};

# async fn demo() -> Result<(), openai_rust::Error> {
let client = OpenAI::from_env()?;
let mut stream = Box::pin(client.files().list_auto_paging(FileListParams::default()));

while let Some(file) = stream.next().await {
    println!("{}", file?.id);
}
# Ok(())
# }
```

## 已实现方法

- `create` / `create_with_options`
- `retrieve` / `retrieve_with_options`
- `list` / `list_with_params` / `list_with_options`
- `list_next_page` / `list_next_page_with_options`
- `list_auto_paging` / `list_auto_paging_with_options`
- `delete` / `delete_with_options`
- `content` / `content_with_options`
