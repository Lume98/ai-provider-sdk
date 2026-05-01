# Files

## 如何使用

### 上传文件

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

### 自动翻页

```rust
use futures_util::StreamExt;
use openai_rust::{FileListParams, OpenAI};

# async fn demo() -> Result<(), openai_rust::Error> {
let client = OpenAI::from_env()?;
let mut stream = std::pin::pin!(client.files().list_auto_paging(FileListParams::default()));

while let Some(file) = stream.next().await {
    println!("{}", file?.id);
}
# Ok(())
# }
```

## 已实现方法

- `create(params)` / `create_with_options(params, options)`
- `retrieve(file_id)` / `retrieve_with_options(file_id, options)`
- `list()` / `list_with_params(params)` / `list_with_options(params, options)`
- `list_next_page(current_page, params)` / `list_next_page_with_options(current_page, params, options)`
- `list_auto_paging(params)` / `list_auto_paging_with_options(params, options)`
- `delete(file_id)` / `delete_with_options(file_id, options)`
- `content(file_id)` / `content_with_options(file_id, options)`

## 入参结构（全量）

`FileCreateParams`

- `file: UploadFile`（必填）上传文件对象。
- `purpose: FilePurpose`（必填）文件用途。
- `expires_after: Option<ExpiresAfter>`（可选）过期策略。
- `extra: HashMap<String, Value>`（可选）扩展字段。

`UploadFile`

- `file_name: String`（必填）multipart 中的文件名。
- `bytes: bytes::Bytes`（必填）文件字节。
- `mime_type: Option<String>`（可选）MIME 类型。

构造方法：

- `UploadFile::from_bytes(file_name, bytes)`
- `UploadFile::from_path(path).await`（路径文件名必须是有效 UTF-8）

`FilePurpose` 枚举值：

- `Assistants`
- `AssistantsOutput`
- `Batch`
- `BatchOutput`
- `FineTune`
- `FineTuneResults`
- `Vision`
- `UserData`
- `Evals`

`ExpiresAfter`

- `anchor: String`（必填）过期锚点。
- `seconds: u32`（必填）相对锚点秒数。

`FileListParams`

- `after: Option<String>`（可选）分页游标。
- `limit: Option<u32>`（可选）单页上限。
- `order: Option<ListOrder>`（可选）排序方向。
- `purpose: Option<String>`（可选）用途过滤。

`ListOrder` 枚举值：

- `Asc`
- `Desc`

## 响应结构（全量）

`FileObject`

- `id: String`
- `bytes: Option<u64>`
- `created_at: Option<u64>`
- `filename: Option<String>`
- `object: Option<String>`
- `purpose: Option<FilePurpose>`
- `status: Option<String>`
- `expires_at: Option<u64>`
- `status_details: Option<String>`
- `extra: HashMap<String, Value>`

`FileDeleted`

- `id: String`
- `deleted: bool`
- `object: Option<String>`
- `extra: HashMap<String, Value>`

`list*` 系列返回分页对象 `CursorPage<FileObject>`，可继续使用 `list_next_page*` 或 `list_auto_paging*`。

## 兼容性说明

- `extra` 为前向兼容容器，不保证稳定结构。
- 文档只覆盖当前仓库已实现能力。
