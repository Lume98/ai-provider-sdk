# 资源总览

当前 SDK 只实现以下资源。未出现在本页的 OpenAI 资源在本仓库中未实现。

## 使用入口

- 安装与使用主线：[/guide/overview](/guide/overview)
- 配置与请求覆盖：[/guide/configuration](/guide/configuration)

## `client.responses()`

- `create(params)`
- `create_with_options(params, options)`
- `create_stream(params)`
- `create_stream_with_options(params, options)`
- 详情：[/api/responses](/api/responses)

## `client.chat().completions()`

- `create(params)`
- `create_with_options(params, options)`
- `create_stream(params)`
- `create_stream_with_options(params, options)`
- 详情：[/api/chat](/api/chat)

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
- 详情：[/api/files](/api/files)

## `client.models()`

- `list()`
- `list_with_options(options)`
- `retrieve(model)`
- `retrieve_with_options(model, options)`
- 详情：[/api/models](/api/models)

## `client.embeddings()`

- `create(params)`
- `create_with_options(params, options)`
- 详情：[/api/embeddings](/api/embeddings)

## `client.moderations()`

- `create(params)`
- `create_with_options(params, options)`
- 详情：[/api/moderations](/api/moderations)
