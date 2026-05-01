# 资源总览

当前 SDK 只实现以下资源，不在列表中的资源均未实现。

## `client.responses()`

- `create(params)`
- `create_with_options(params, options)`
- `create_stream(params)`
- `create_stream_with_options(params, options)`

## `client.chat().completions()`

- `create(params)`
- `create_with_options(params, options)`
- `create_stream(params)`
- `create_stream_with_options(params, options)`

## `client.files()`

- `create(params)`
- `create_with_options(params, options)`
- `retrieve(file_id)`
- `retrieve_with_options(file_id, options)`
- `list()`
- `list_with_params(params)`
- `list_with_options(params, options)`
- `list_next_page(current_page, params)`
- `list_next_page_with_options(current_page, params, options)`
- `list_auto_paging(params)`
- `list_auto_paging_with_options(params, options)`
- `delete(file_id)`
- `delete_with_options(file_id, options)`
- `content(file_id)`
- `content_with_options(file_id, options)`

## `client.models()`

- `list()`
- `list_with_options(options)`
- `retrieve(model)`
- `retrieve_with_options(model, options)`

## `client.embeddings()`

- `create(params)`
- `create_with_options(params, options)`

## `client.moderations()`

- `create(params)`
- `create_with_options(params, options)`
