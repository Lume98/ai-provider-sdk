# Rust SDK 支持进度

更新时间：2026-04-30

说明：下表按本仓库 `OpenAIClient` 的实际实现统计。

| 资源 | 状态 | 当前支持 |
|---|---|---|
| `client.responses` | `已支持` | `create` / `stream` / `retrieve` / `delete` |
| `client.chat.completions` | `部分支持` | `create` / `create_stream`（未实现 `retrieve` / `list` / `update` / `delete`） |
| `client.files` | `已支持` | `create` / `list` / `retrieve` / `delete` / `content` |
| `client.uploads` | `已支持` | `create` / `add_part` / `complete` / `cancel` |
| `client.models` | `部分支持` | `list` / `retrieve` / `delete`（未实现 Python SDK 中的权限相关接口） |
| `client.completions` | `已支持` | `create` |
| `client.embeddings` | `已支持` | `create` |
| `client.moderations` | `已支持` | `create` |
| `client.images` | `部分支持` | `generate` / `edit`（未实现 `create_variation`） |
| `client.audio` | `已支持` | `speech` / `transcriptions` / `translations` |
| `client.batches` | `部分支持` | `create` / `retrieve` / `list` / `delete`（未实现 `cancel`） |
| `client.fine_tuning` | `部分支持` | `create_job` / `retrieve_job` / `list_jobs` / `cancel_job` |
| `client.evals` | `部分支持` | `create` / `retrieve` / `list` / `delete`（未实现 `runs` 子资源） |
| `client.containers` | `部分支持` | `create` / `retrieve` / `list` / `delete`（未实现 `files` 子资源） |
| `client.conversations` | `部分支持` | `create` / `retrieve` / `list` / `delete` |
| `client.vector_stores` | `部分支持` | `create` / `retrieve` / `list` / `delete` |
| `client.skills` | `部分支持` | `create` / `retrieve` / `list` / `delete` |
| `client.realtime` | `部分支持` | `create_session` / `create_transcription_session`（`connect` 当前返回 `Unsupported`） |
| `client.beta` | `部分支持` | `create_assistant` / `retrieve_assistant` |
| `client.webhooks` | `已支持` | `verify_signature`（本地 helper） |

- 统计口径：`client.*` 方法（含 HTTP 接口与 SDK helper 方法）
- `HTTP = -` 且路径为 `(helper/no-http)` 的条目表示本地封装的便捷方法，不对应单独 HTTP endpoint
