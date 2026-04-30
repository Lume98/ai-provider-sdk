# 资源总览

本页基于上一级 `openai-python/api.md` 自动提取，覆盖 OpenAI Python SDK 当前公开接口。

- 统计口径：`client.*` 方法（含 HTTP 接口与 SDK helper 方法）
- `HTTP = -` 且路径为 `(helper/no-http)` 的条目表示本地封装的便捷方法，不对应单独 HTTP endpoint

## `client.audio.speech`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/audio/speech` |

## `client.audio.transcriptions`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/audio/transcriptions` |

## `client.audio.translations`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/audio/translations` |

## `client.batches`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/batches` |
| `retrieve` | `GET` | `/batches/{batch_id}` |
| `list` | `GET` | `/batches` |
| `cancel` | `POST` | `/batches/{batch_id}/cancel` |

## `client.beta.assistants`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/assistants` |
| `retrieve` | `GET` | `/assistants/{assistant_id}` |
| `update` | `POST` | `/assistants/{assistant_id}` |
| `list` | `GET` | `/assistants` |
| `delete` | `DELETE` | `/assistants/{assistant_id}` |

## `client.beta.realtime.sessions`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/realtime/sessions` |

## `client.beta.realtime.transcription_sessions`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/realtime/transcription_sessions` |

## `client.beta.threads`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/threads` |
| `retrieve` | `GET` | `/threads/{thread_id}` |
| `update` | `POST` | `/threads/{thread_id}` |
| `delete` | `DELETE` | `/threads/{thread_id}` |
| `create_and_run` | `POST` | `/threads/runs` |
| `create_and_run_poll` | `-` | `(helper/no-http)` |
| `create_and_run_stream` | `-` | `(helper/no-http)` |

## `client.beta.threads.messages`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/threads/{thread_id}/messages` |
| `retrieve` | `GET` | `/threads/{thread_id}/messages/{message_id}` |
| `update` | `POST` | `/threads/{thread_id}/messages/{message_id}` |
| `list` | `GET` | `/threads/{thread_id}/messages` |
| `delete` | `DELETE` | `/threads/{thread_id}/messages/{message_id}` |

## `client.beta.threads.runs`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/threads/{thread_id}/runs` |
| `retrieve` | `GET` | `/threads/{thread_id}/runs/{run_id}` |
| `update` | `POST` | `/threads/{thread_id}/runs/{run_id}` |
| `list` | `GET` | `/threads/{thread_id}/runs` |
| `cancel` | `POST` | `/threads/{thread_id}/runs/{run_id}/cancel` |
| `submit_tool_outputs` | `POST` | `/threads/{thread_id}/runs/{run_id}/submit_tool_outputs` |
| `create_and_poll` | `-` | `(helper/no-http)` |
| `create_and_stream` | `-` | `(helper/no-http)` |
| `poll` | `-` | `(helper/no-http)` |
| `stream` | `-` | `(helper/no-http)` |
| `submit_tool_outputs_and_poll` | `-` | `(helper/no-http)` |
| `submit_tool_outputs_stream` | `-` | `(helper/no-http)` |

## `client.beta.threads.runs.steps`

| 方法 | HTTP | 路径 |
|---|---|---|
| `retrieve` | `GET` | `/threads/{thread_id}/runs/{run_id}/steps/{step_id}` |
| `list` | `GET` | `/threads/{thread_id}/runs/{run_id}/steps` |

## `client.chat.completions`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/chat/completions` |
| `retrieve` | `GET` | `/chat/completions/{completion_id}` |
| `update` | `POST` | `/chat/completions/{completion_id}` |
| `list` | `GET` | `/chat/completions` |
| `delete` | `DELETE` | `/chat/completions/{completion_id}` |

## `client.chat.completions.messages`

| 方法 | HTTP | 路径 |
|---|---|---|
| `list` | `GET` | `/chat/completions/{completion_id}/messages` |

## `client.completions`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/completions` |

## `client.containers`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/containers` |
| `retrieve` | `GET` | `/containers/{container_id}` |
| `list` | `GET` | `/containers` |
| `delete` | `DELETE` | `/containers/{container_id}` |

## `client.containers.files`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/containers/{container_id}/files` |
| `retrieve` | `GET` | `/containers/{container_id}/files/{file_id}` |
| `list` | `GET` | `/containers/{container_id}/files` |
| `delete` | `DELETE` | `/containers/{container_id}/files/{file_id}` |

## `client.containers.files.content`

| 方法 | HTTP | 路径 |
|---|---|---|
| `retrieve` | `GET` | `/containers/{container_id}/files/{file_id}/content` |

## `client.embeddings`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/embeddings` |

## `client.evals`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/evals` |
| `retrieve` | `GET` | `/evals/{eval_id}` |
| `update` | `POST` | `/evals/{eval_id}` |
| `list` | `GET` | `/evals` |
| `delete` | `DELETE` | `/evals/{eval_id}` |

## `client.evals.runs`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/evals/{eval_id}/runs` |
| `retrieve` | `GET` | `/evals/{eval_id}/runs/{run_id}` |
| `list` | `GET` | `/evals/{eval_id}/runs` |
| `delete` | `DELETE` | `/evals/{eval_id}/runs/{run_id}` |
| `cancel` | `POST` | `/evals/{eval_id}/runs/{run_id}` |

## `client.evals.runs.output_items`

| 方法 | HTTP | 路径 |
|---|---|---|
| `retrieve` | `GET` | `/evals/{eval_id}/runs/{run_id}/output_items/{output_item_id}` |
| `list` | `GET` | `/evals/{eval_id}/runs/{run_id}/output_items` |

## `client.files`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/files` |
| `retrieve` | `GET` | `/files/{file_id}` |
| `list` | `GET` | `/files` |
| `delete` | `DELETE` | `/files/{file_id}` |
| `content` | `GET` | `/files/{file_id}/content` |
| `retrieve_content` | `GET` | `/files/{file_id}/content` |
| `wait_for_processing` | `-` | `(helper/no-http)` |

## `client.fine_tuning.alpha.graders`

| 方法 | HTTP | 路径 |
|---|---|---|
| `run` | `POST` | `/fine_tuning/alpha/graders/run` |
| `validate` | `POST` | `/fine_tuning/alpha/graders/validate` |

## `client.fine_tuning.checkpoints.permissions`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions` |
| `retrieve` | `GET` | `/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions` |
| `list` | `GET` | `/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions` |
| `delete` | `DELETE` | `/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions/{permission_id}` |

## `client.fine_tuning.jobs`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/fine_tuning/jobs` |
| `retrieve` | `GET` | `/fine_tuning/jobs/{fine_tuning_job_id}` |
| `list` | `GET` | `/fine_tuning/jobs` |
| `cancel` | `POST` | `/fine_tuning/jobs/{fine_tuning_job_id}/cancel` |
| `list_events` | `GET` | `/fine_tuning/jobs/{fine_tuning_job_id}/events` |
| `pause` | `POST` | `/fine_tuning/jobs/{fine_tuning_job_id}/pause` |
| `resume` | `POST` | `/fine_tuning/jobs/{fine_tuning_job_id}/resume` |

## `client.fine_tuning.jobs.checkpoints`

| 方法 | HTTP | 路径 |
|---|---|---|
| `list` | `GET` | `/fine_tuning/jobs/{fine_tuning_job_id}/checkpoints` |

## `client.images`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create_variation` | `POST` | `/images/variations` |
| `edit` | `POST` | `/images/edits` |
| `generate` | `POST` | `/images/generations` |

## `client.models`

| 方法 | HTTP | 路径 |
|---|---|---|
| `retrieve` | `GET` | `/models/{model}` |
| `list` | `GET` | `/models` |
| `delete` | `DELETE` | `/models/{model}` |

## `client.moderations`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/moderations` |

## `client.skills`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/skills` |
| `retrieve` | `GET` | `/skills/{skill_id}` |
| `update` | `POST` | `/skills/{skill_id}` |
| `list` | `GET` | `/skills` |
| `delete` | `DELETE` | `/skills/{skill_id}` |

## `client.skills.content`

| 方法 | HTTP | 路径 |
|---|---|---|
| `retrieve` | `GET` | `/skills/{skill_id}/content` |

## `client.skills.versions`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/skills/{skill_id}/versions` |
| `retrieve` | `GET` | `/skills/{skill_id}/versions/{version}` |
| `list` | `GET` | `/skills/{skill_id}/versions` |
| `delete` | `DELETE` | `/skills/{skill_id}/versions/{version}` |

## `client.skills.versions.content`

| 方法 | HTTP | 路径 |
|---|---|---|
| `retrieve` | `GET` | `/skills/{skill_id}/versions/{version}/content` |

## `client.uploads`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/uploads` |
| `cancel` | `POST` | `/uploads/{upload_id}/cancel` |
| `complete` | `POST` | `/uploads/{upload_id}/complete` |

## `client.uploads.parts`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/uploads/{upload_id}/parts` |

## `client.vector_stores`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/vector_stores` |
| `retrieve` | `GET` | `/vector_stores/{vector_store_id}` |
| `update` | `POST` | `/vector_stores/{vector_store_id}` |
| `list` | `GET` | `/vector_stores` |
| `delete` | `DELETE` | `/vector_stores/{vector_store_id}` |
| `search` | `POST` | `/vector_stores/{vector_store_id}/search` |

## `client.vector_stores.file_batches`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/vector_stores/{vector_store_id}/file_batches` |
| `retrieve` | `GET` | `/vector_stores/{vector_store_id}/file_batches/{batch_id}` |
| `cancel` | `POST` | `/vector_stores/{vector_store_id}/file_batches/{batch_id}/cancel` |
| `list_files` | `GET` | `/vector_stores/{vector_store_id}/file_batches/{batch_id}/files` |
| `create_and_poll` | `-` | `(helper/no-http)` |
| `poll` | `-` | `(helper/no-http)` |
| `upload_and_poll` | `-` | `(helper/no-http)` |

## `client.vector_stores.files`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/vector_stores/{vector_store_id}/files` |
| `retrieve` | `GET` | `/vector_stores/{vector_store_id}/files/{file_id}` |
| `update` | `POST` | `/vector_stores/{vector_store_id}/files/{file_id}` |
| `list` | `GET` | `/vector_stores/{vector_store_id}/files` |
| `delete` | `DELETE` | `/vector_stores/{vector_store_id}/files/{file_id}` |
| `content` | `GET` | `/vector_stores/{vector_store_id}/files/{file_id}/content` |
| `create_and_poll` | `-` | `(helper/no-http)` |
| `poll` | `-` | `(helper/no-http)` |
| `upload` | `-` | `(helper/no-http)` |
| `upload_and_poll` | `-` | `(helper/no-http)` |

## `client.videos`

| 方法 | HTTP | 路径 |
|---|---|---|
| `create` | `POST` | `/videos` |
| `retrieve` | `GET` | `/videos/{video_id}` |
| `list` | `GET` | `/videos` |
| `delete` | `DELETE` | `/videos/{video_id}` |
| `create_character` | `POST` | `/videos/characters` |
| `download_content` | `GET` | `/videos/{video_id}/content` |
| `edit` | `POST` | `/videos/edits` |
| `extend` | `POST` | `/videos/extensions` |
| `get_character` | `GET` | `/videos/characters/{character_id}` |
| `remix` | `POST` | `/videos/{video_id}/remix` |
| `create_and_poll` | `-` | `(helper/no-http)` |
| `poll` | `-` | `(helper/no-http)` |

## `client.webhooks`

| 方法 | HTTP | 路径 |
|---|---|---|
| `unwrap` | `-` | `(helper/no-http)` |
| `verify_signature` | `-` | `(helper/no-http)` |

