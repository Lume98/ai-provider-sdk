# 资源总览

`OpenAIClient` 当前暴露以下资源入口：

- `client.responses.create/stream/retrieve/delete`
- `client.chat.completions.create/create_stream`
- `client.completions.create`
- `client.models.list/retrieve/delete`
- `client.files.create/retrieve/list/delete/content`
- `client.uploads.create/add_part/complete/cancel`
- `client.images.generate/edit`
- `client.audio.speech/transcriptions/translations`
- `client.embeddings.create`
- `client.moderations.create`
- `client.batches`、`client.fine_tuning`、`client.evals`、`client.containers`
- `client.conversations`、`client.vector_stores`、`client.realtime`、`client.webhooks`、`client.skills`、`client.beta`

其中一部分资源（如 batches / conversations / skills）通过通用对象建模，主要使用 `GenericCreateParams` 和 `GenericObject` 承载扩展字段。
