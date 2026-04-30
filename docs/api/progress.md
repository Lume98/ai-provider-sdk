# Rust SDK 支持进度

更新时间：2026-04-30

说明：本文件由 `codegen/generate_openai.py` 从本地 `openai-python` 生成产物自动生成。
当前阶段是覆盖矩阵，不代表所有缺口都已有 Rust 生成实现。

## 汇总

- Python resources: `51`
- Python public methods: `184`
- Python type files: `910`
- Methods currently matched by hand-written Rust SDK: `74`
- Methods covered by Python api_resource tests: `144`

## Resource Matrix

| Resource | Python methods | Rust matched | Missing | Python tests |
|---|---:|---:|---:|---:|
| `audio.speech` | `1` | `1` | `0` | `1` |
| `audio.transcriptions` | `1` | `1` | `0` | `0` |
| `audio.translations` | `1` | `1` | `0` | `1` |
| `batches` | `4` | `4` | `0` | `4` |
| `beta.assistants` | `5` | `2` | `3` | `5` |
| `beta.chatkit.sessions` | `2` | `0` | `2` | `2` |
| `beta.chatkit.threads` | `4` | `0` | `4` | `4` |
| `beta.realtime` | `1` | `0` | `1` | `0` |
| `beta.realtime.sessions` | `1` | `0` | `1` | `0` |
| `beta.realtime.transcription_sessions` | `1` | `0` | `1` | `0` |
| `beta.threads` | `7` | `0` | `7` | `4` |
| `beta.threads.messages` | `5` | `0` | `5` | `5` |
| `beta.threads.runs` | `12` | `0` | `12` | `4` |
| `beta.threads.runs.steps` | `2` | `0` | `2` | `2` |
| `chat.completions` | `7` | `5` | `2` | `4` |
| `chat.completions.messages` | `1` | `1` | `0` | `1` |
| `completions` | `1` | `1` | `0` | `0` |
| `containers` | `4` | `4` | `0` | `4` |
| `containers.files` | `4` | `0` | `4` | `4` |
| `containers.files.content` | `1` | `0` | `1` | `1` |
| `conversations` | `4` | `4` | `0` | `4` |
| `conversations.items` | `4` | `0` | `4` | `4` |
| `embeddings` | `1` | `1` | `0` | `1` |
| `evals` | `5` | `5` | `0` | `5` |
| `evals.runs` | `5` | `0` | `5` | `5` |
| `evals.runs.output_items` | `2` | `0` | `2` | `2` |
| `files` | `7` | `5` | `2` | `6` |
| `fine_tuning.alpha.graders` | `2` | `0` | `2` | `2` |
| `fine_tuning.checkpoints.permissions` | `4` | `0` | `4` | `4` |
| `fine_tuning.jobs` | `7` | `7` | `0` | `7` |
| `fine_tuning.jobs.checkpoints` | `1` | `0` | `1` | `1` |
| `images` | `3` | `3` | `0` | `1` |
| `models` | `3` | `3` | `0` | `3` |
| `moderations` | `1` | `1` | `0` | `1` |
| `realtime` | `1` | `1` | `0` | `0` |
| `realtime.calls` | `5` | `0` | `5` | `5` |
| `realtime.client_secrets` | `1` | `0` | `1` | `1` |
| `responses` | `8` | `6` | `2` | `3` |
| `responses.input_items` | `1` | `1` | `0` | `1` |
| `responses.input_tokens` | `1` | `1` | `0` | `1` |
| `skills` | `5` | `5` | `0` | `5` |
| `skills.content` | `1` | `0` | `1` | `1` |
| `skills.versions` | `4` | `0` | `4` | `4` |
| `skills.versions.content` | `1` | `0` | `1` | `1` |
| `uploads` | `4` | `3` | `1` | `3` |
| `uploads.parts` | `1` | `1` | `0` | `1` |
| `vector_stores` | `6` | `6` | `0` | `6` |
| `vector_stores.file_batches` | `7` | `0` | `7` | `4` |
| `vector_stores.files` | `10` | `0` | `10` | `6` |
| `videos` | `12` | `0` | `12` | `10` |
| `webhooks` | `2` | `1` | `1` | `0` |

## Method Matrix

| Resource | Method | HTTP | Path | Rust | Python test |
|---|---|---|---|---|---|
| `audio.speech` | `create` | `POST` | `/audio/speech` | `create` | `yes` |
| `audio.transcriptions` | `create` | `POST` | `/audio/transcriptions` | `create` | `no` |
| `audio.translations` | `create` | `POST` | `/audio/translations` | `create` | `yes` |
| `batches` | `create` | `POST` | `/batches` | `create` | `yes` |
| `batches` | `retrieve` | `GET` | `/batches/{batch_id}` | `retrieve` | `yes` |
| `batches` | `list` | `GET` | `/batches` | `list` | `yes` |
| `batches` | `cancel` | `POST` | `/batches/{batch_id}/cancel` | `cancel` | `yes` |
| `beta.assistants` | `create` | `POST` | `/assistants` | `create` | `yes` |
| `beta.assistants` | `retrieve` | `GET` | `/assistants/{assistant_id}` | `retrieve` | `yes` |
| `beta.assistants` | `update` | `POST` | `/assistants/{assistant_id}` | `missing` | `yes` |
| `beta.assistants` | `list` | `GET` | `/assistants` | `missing` | `yes` |
| `beta.assistants` | `delete` | `DELETE` | `/assistants/{assistant_id}` | `missing` | `yes` |
| `beta.chatkit.sessions` | `create` | `POST` | `/chatkit/sessions` | `missing` | `yes` |
| `beta.chatkit.sessions` | `cancel` | `POST` | `/chatkit/sessions/{session_id}/cancel` | `missing` | `yes` |
| `beta.chatkit.threads` | `retrieve` | `GET` | `/chatkit/threads/{thread_id}` | `missing` | `yes` |
| `beta.chatkit.threads` | `list` | `GET` | `/chatkit/threads` | `missing` | `yes` |
| `beta.chatkit.threads` | `delete` | `DELETE` | `/chatkit/threads/{thread_id}` | `missing` | `yes` |
| `beta.chatkit.threads` | `list_items` | `GET` | `/chatkit/threads/{thread_id}/items` | `missing` | `yes` |
| `beta.realtime` | `connect` | `-` | `(helper/no-http)` | `missing` | `no` |
| `beta.realtime.sessions` | `create` | `POST` | `/realtime/sessions` | `missing` | `no` |
| `beta.realtime.transcription_sessions` | `create` | `POST` | `/realtime/transcription_sessions` | `missing` | `no` |
| `beta.threads` | `create` | `POST` | `/threads` | `missing` | `yes` |
| `beta.threads` | `retrieve` | `GET` | `/threads/{thread_id}` | `missing` | `yes` |
| `beta.threads` | `update` | `POST` | `/threads/{thread_id}` | `missing` | `yes` |
| `beta.threads` | `delete` | `DELETE` | `/threads/{thread_id}` | `missing` | `yes` |
| `beta.threads` | `create_and_run` | `POST` | `/threads/runs` | `missing` | `no` |
| `beta.threads` | `create_and_run_poll` | `-` | `(helper/no-http)` | `missing` | `no` |
| `beta.threads` | `create_and_run_stream` | `-` | `(helper/no-http)` | `missing` | `no` |
| `beta.threads.messages` | `create` | `POST` | `/threads/{thread_id}/messages` | `missing` | `yes` |
| `beta.threads.messages` | `retrieve` | `GET` | `/threads/{thread_id}/messages/{message_id}` | `missing` | `yes` |
| `beta.threads.messages` | `update` | `POST` | `/threads/{thread_id}/messages/{message_id}` | `missing` | `yes` |
| `beta.threads.messages` | `list` | `GET` | `/threads/{thread_id}/messages` | `missing` | `yes` |
| `beta.threads.messages` | `delete` | `DELETE` | `/threads/{thread_id}/messages/{message_id}` | `missing` | `yes` |
| `beta.threads.runs` | `create` | `POST` | `/threads/{thread_id}/runs` | `missing` | `no` |
| `beta.threads.runs` | `retrieve` | `GET` | `/threads/{thread_id}/runs/{run_id}` | `missing` | `yes` |
| `beta.threads.runs` | `update` | `POST` | `/threads/{thread_id}/runs/{run_id}` | `missing` | `yes` |
| `beta.threads.runs` | `list` | `GET` | `/threads/{thread_id}/runs` | `missing` | `yes` |
| `beta.threads.runs` | `cancel` | `POST` | `/threads/{thread_id}/runs/{run_id}/cancel` | `missing` | `yes` |
| `beta.threads.runs` | `create_and_poll` | `-` | `(helper/no-http)` | `missing` | `no` |
| `beta.threads.runs` | `create_and_stream` | `-` | `(helper/no-http)` | `missing` | `no` |
| `beta.threads.runs` | `poll` | `-` | `(helper/no-http)` | `missing` | `no` |
| `beta.threads.runs` | `stream` | `-` | `(helper/no-http)` | `missing` | `no` |
| `beta.threads.runs` | `submit_tool_outputs` | `POST` | `/threads/{thread_id}/runs/{run_id}/submit_tool_outputs` | `missing` | `no` |
| `beta.threads.runs` | `submit_tool_outputs_and_poll` | `-` | `(helper/no-http)` | `missing` | `no` |
| `beta.threads.runs` | `submit_tool_outputs_stream` | `-` | `(helper/no-http)` | `missing` | `no` |
| `beta.threads.runs.steps` | `retrieve` | `GET` | `/threads/{thread_id}/runs/{run_id}/steps/{step_id}` | `missing` | `yes` |
| `beta.threads.runs.steps` | `list` | `GET` | `/threads/{thread_id}/runs/{run_id}/steps` | `missing` | `yes` |
| `chat.completions` | `parse` | `POST` | `/chat/completions` | `missing` | `no` |
| `chat.completions` | `create` | `POST` | `/chat/completions` | `create` | `no` |
| `chat.completions` | `retrieve` | `GET` | `/chat/completions/{completion_id}` | `retrieve` | `yes` |
| `chat.completions` | `update` | `POST` | `/chat/completions/{completion_id}` | `update` | `yes` |
| `chat.completions` | `list` | `GET` | `/chat/completions` | `list` | `yes` |
| `chat.completions` | `delete` | `DELETE` | `/chat/completions/{completion_id}` | `delete` | `yes` |
| `chat.completions` | `stream` | `-` | `(helper/no-http)` | `missing` | `no` |
| `chat.completions.messages` | `list` | `GET` | `/chat/completions/{completion_id}/messages` | `list` | `yes` |
| `completions` | `create` | `POST` | `/completions` | `create` | `no` |
| `containers` | `create` | `POST` | `/containers` | `create` | `yes` |
| `containers` | `retrieve` | `GET` | `/containers/{container_id}` | `retrieve` | `yes` |
| `containers` | `list` | `GET` | `/containers` | `list` | `yes` |
| `containers` | `delete` | `DELETE` | `/containers/{container_id}` | `delete` | `yes` |
| `containers.files` | `create` | `POST` | `/containers/{container_id}/files` | `missing` | `yes` |
| `containers.files` | `retrieve` | `GET` | `/containers/{container_id}/files/{file_id}` | `missing` | `yes` |
| `containers.files` | `list` | `GET` | `/containers/{container_id}/files` | `missing` | `yes` |
| `containers.files` | `delete` | `DELETE` | `/containers/{container_id}/files/{file_id}` | `missing` | `yes` |
| `containers.files.content` | `retrieve` | `GET` | `/containers/{container_id}/files/{file_id}/content` | `missing` | `yes` |
| `conversations` | `create` | `POST` | `/conversations` | `create` | `yes` |
| `conversations` | `retrieve` | `GET` | `/conversations/{conversation_id}` | `retrieve` | `yes` |
| `conversations` | `update` | `POST` | `/conversations/{conversation_id}` | `update` | `yes` |
| `conversations` | `delete` | `DELETE` | `/conversations/{conversation_id}` | `delete` | `yes` |
| `conversations.items` | `create` | `POST` | `/conversations/{conversation_id}/items` | `missing` | `yes` |
| `conversations.items` | `retrieve` | `GET` | `/conversations/{conversation_id}/items/{item_id}` | `missing` | `yes` |
| `conversations.items` | `list` | `GET` | `/conversations/{conversation_id}/items` | `missing` | `yes` |
| `conversations.items` | `delete` | `DELETE` | `/conversations/{conversation_id}/items/{item_id}` | `missing` | `yes` |
| `embeddings` | `create` | `POST` | `/embeddings` | `create` | `yes` |
| `evals` | `create` | `POST` | `/evals` | `create` | `yes` |
| `evals` | `retrieve` | `GET` | `/evals/{eval_id}` | `retrieve` | `yes` |
| `evals` | `update` | `POST` | `/evals/{eval_id}` | `update` | `yes` |
| `evals` | `list` | `GET` | `/evals` | `list` | `yes` |
| `evals` | `delete` | `DELETE` | `/evals/{eval_id}` | `delete` | `yes` |
| `evals.runs` | `create` | `POST` | `/evals/{eval_id}/runs` | `missing` | `yes` |
| `evals.runs` | `retrieve` | `GET` | `/evals/{eval_id}/runs/{run_id}` | `missing` | `yes` |
| `evals.runs` | `list` | `GET` | `/evals/{eval_id}/runs` | `missing` | `yes` |
| `evals.runs` | `delete` | `DELETE` | `/evals/{eval_id}/runs/{run_id}` | `missing` | `yes` |
| `evals.runs` | `cancel` | `POST` | `/evals/{eval_id}/runs/{run_id}` | `missing` | `yes` |
| `evals.runs.output_items` | `retrieve` | `GET` | `/evals/{eval_id}/runs/{run_id}/output_items/{output_item_id}` | `missing` | `yes` |
| `evals.runs.output_items` | `list` | `GET` | `/evals/{eval_id}/runs/{run_id}/output_items` | `missing` | `yes` |
| `files` | `create` | `POST` | `/files` | `create` | `yes` |
| `files` | `retrieve` | `GET` | `/files/{file_id}` | `retrieve` | `yes` |
| `files` | `list` | `GET` | `/files` | `list` | `yes` |
| `files` | `delete` | `DELETE` | `/files/{file_id}` | `delete` | `yes` |
| `files` | `content` | `GET` | `/files/{file_id}/content` | `content` | `yes` |
| `files` | `retrieve_content` | `GET` | `/files/{file_id}/content` | `missing` | `yes` |
| `files` | `wait_for_processing` | `-` | `(helper/no-http)` | `missing` | `no` |
| `fine_tuning.alpha.graders` | `run` | `POST` | `/fine_tuning/alpha/graders/run` | `missing` | `yes` |
| `fine_tuning.alpha.graders` | `validate` | `POST` | `/fine_tuning/alpha/graders/validate` | `missing` | `yes` |
| `fine_tuning.checkpoints.permissions` | `create` | `GET` | `/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions` | `missing` | `yes` |
| `fine_tuning.checkpoints.permissions` | `retrieve` | `GET` | `/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions` | `missing` | `yes` |
| `fine_tuning.checkpoints.permissions` | `list` | `GET` | `/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions` | `missing` | `yes` |
| `fine_tuning.checkpoints.permissions` | `delete` | `DELETE` | `/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions/{permission_id}` | `missing` | `yes` |
| `fine_tuning.jobs` | `create` | `POST` | `/fine_tuning/jobs` | `create` | `yes` |
| `fine_tuning.jobs` | `retrieve` | `GET` | `/fine_tuning/jobs/{fine_tuning_job_id}` | `retrieve` | `yes` |
| `fine_tuning.jobs` | `list` | `GET` | `/fine_tuning/jobs` | `list` | `yes` |
| `fine_tuning.jobs` | `cancel` | `POST` | `/fine_tuning/jobs/{fine_tuning_job_id}/cancel` | `cancel` | `yes` |
| `fine_tuning.jobs` | `list_events` | `GET` | `/fine_tuning/jobs/{fine_tuning_job_id}/events` | `list_events` | `yes` |
| `fine_tuning.jobs` | `pause` | `POST` | `/fine_tuning/jobs/{fine_tuning_job_id}/pause` | `pause` | `yes` |
| `fine_tuning.jobs` | `resume` | `POST` | `/fine_tuning/jobs/{fine_tuning_job_id}/resume` | `resume` | `yes` |
| `fine_tuning.jobs.checkpoints` | `list` | `GET` | `/fine_tuning/jobs/{fine_tuning_job_id}/checkpoints` | `missing` | `yes` |
| `images` | `create_variation` | `POST` | `/images/variations` | `create_variation` | `yes` |
| `images` | `edit` | `POST` | `/images/edits` | `edit` | `no` |
| `images` | `generate` | `POST` | `/images/generations` | `generate` | `no` |
| `models` | `retrieve` | `GET` | `/models/{model}` | `retrieve` | `yes` |
| `models` | `list` | `GET` | `/models` | `list` | `yes` |
| `models` | `delete` | `DELETE` | `/models/{model}` | `delete` | `yes` |
| `moderations` | `create` | `POST` | `/moderations` | `create` | `yes` |
| `realtime` | `connect` | `-` | `(helper/no-http)` | `connect` | `no` |
| `realtime.calls` | `create` | `POST` | `/realtime/calls` | `missing` | `yes` |
| `realtime.calls` | `accept` | `POST` | `/realtime/calls/{call_id}/accept` | `missing` | `yes` |
| `realtime.calls` | `hangup` | `POST` | `/realtime/calls/{call_id}/hangup` | `missing` | `yes` |
| `realtime.calls` | `refer` | `POST` | `/realtime/calls/{call_id}/refer` | `missing` | `yes` |
| `realtime.calls` | `reject` | `POST` | `/realtime/calls/{call_id}/reject` | `missing` | `yes` |
| `realtime.client_secrets` | `create` | `POST` | `/realtime/client_secrets` | `missing` | `yes` |
| `responses` | `create` | `POST` | `/responses` | `create` | `no` |
| `responses` | `stream` | `-` | `(helper/no-http)` | `stream` | `no` |
| `responses` | `parse` | `POST` | `/responses` | `missing` | `no` |
| `responses` | `retrieve` | `GET` | `/responses/{response_id}` | `retrieve` | `no` |
| `responses` | `delete` | `DELETE` | `/responses/{response_id}` | `delete` | `yes` |
| `responses` | `cancel` | `POST` | `/responses/{response_id}/cancel` | `cancel` | `yes` |
| `responses` | `compact` | `POST` | `/responses/compact` | `compact` | `yes` |
| `responses` | `connect` | `-` | `(helper/no-http)` | `missing` | `no` |
| `responses.input_items` | `list` | `GET` | `/responses/{response_id}/input_items` | `list` | `yes` |
| `responses.input_tokens` | `count` | `POST` | `/responses/input_tokens` | `count` | `yes` |
| `skills` | `create` | `POST` | `/skills` | `create` | `yes` |
| `skills` | `retrieve` | `GET` | `/skills/{skill_id}` | `retrieve` | `yes` |
| `skills` | `update` | `POST` | `/skills/{skill_id}` | `update` | `yes` |
| `skills` | `list` | `GET` | `/skills` | `list` | `yes` |
| `skills` | `delete` | `DELETE` | `/skills/{skill_id}` | `delete` | `yes` |
| `skills.content` | `retrieve` | `GET` | `/skills/{skill_id}/content` | `missing` | `yes` |
| `skills.versions` | `create` | `POST` | `/skills/{skill_id}/versions` | `missing` | `yes` |
| `skills.versions` | `retrieve` | `GET` | `/skills/{skill_id}/versions/{version}` | `missing` | `yes` |
| `skills.versions` | `list` | `GET` | `/skills/{skill_id}/versions` | `missing` | `yes` |
| `skills.versions` | `delete` | `DELETE` | `/skills/{skill_id}/versions/{version}` | `missing` | `yes` |
| `skills.versions.content` | `retrieve` | `GET` | `/skills/{skill_id}/versions/{version}/content` | `missing` | `yes` |
| `uploads` | `upload_file_chunked` | `-` | `(helper/no-http)` | `missing` | `no` |
| `uploads` | `create` | `POST` | `/uploads` | `create` | `yes` |
| `uploads` | `cancel` | `POST` | `/uploads/{upload_id}/cancel` | `cancel` | `yes` |
| `uploads` | `complete` | `POST` | `/uploads/{upload_id}/complete` | `complete` | `yes` |
| `uploads.parts` | `create` | `POST` | `/uploads/{upload_id}/parts` | `create` | `yes` |
| `vector_stores` | `create` | `POST` | `/vector_stores` | `create` | `yes` |
| `vector_stores` | `retrieve` | `GET` | `/vector_stores/{vector_store_id}` | `retrieve` | `yes` |
| `vector_stores` | `update` | `POST` | `/vector_stores/{vector_store_id}` | `update` | `yes` |
| `vector_stores` | `list` | `GET` | `/vector_stores` | `list` | `yes` |
| `vector_stores` | `delete` | `DELETE` | `/vector_stores/{vector_store_id}` | `delete` | `yes` |
| `vector_stores` | `search` | `GET` | `/vector_stores/{vector_store_id}/search` | `search` | `yes` |
| `vector_stores.file_batches` | `create` | `POST` | `/vector_stores/{vector_store_id}/file_batches` | `missing` | `yes` |
| `vector_stores.file_batches` | `retrieve` | `GET` | `/vector_stores/{vector_store_id}/file_batches/{batch_id}` | `missing` | `yes` |
| `vector_stores.file_batches` | `cancel` | `POST` | `/vector_stores/{vector_store_id}/file_batches/{batch_id}/cancel` | `missing` | `yes` |
| `vector_stores.file_batches` | `create_and_poll` | `-` | `(helper/no-http)` | `missing` | `no` |
| `vector_stores.file_batches` | `list_files` | `GET` | `/vector_stores/{vector_store_id}/file_batches/{batch_id}/files` | `missing` | `yes` |
| `vector_stores.file_batches` | `poll` | `-` | `(helper/no-http)` | `missing` | `no` |
| `vector_stores.file_batches` | `upload_and_poll` | `-` | `(helper/no-http)` | `missing` | `no` |
| `vector_stores.files` | `create` | `POST` | `/vector_stores/{vector_store_id}/files` | `missing` | `yes` |
| `vector_stores.files` | `retrieve` | `GET` | `/vector_stores/{vector_store_id}/files/{file_id}` | `missing` | `yes` |
| `vector_stores.files` | `update` | `POST` | `/vector_stores/{vector_store_id}/files/{file_id}` | `missing` | `yes` |
| `vector_stores.files` | `list` | `GET` | `/vector_stores/{vector_store_id}/files` | `missing` | `yes` |
| `vector_stores.files` | `delete` | `DELETE` | `/vector_stores/{vector_store_id}/files/{file_id}` | `missing` | `yes` |
| `vector_stores.files` | `create_and_poll` | `-` | `(helper/no-http)` | `missing` | `no` |
| `vector_stores.files` | `poll` | `-` | `(helper/no-http)` | `missing` | `no` |
| `vector_stores.files` | `upload` | `-` | `(helper/no-http)` | `missing` | `no` |
| `vector_stores.files` | `upload_and_poll` | `-` | `(helper/no-http)` | `missing` | `no` |
| `vector_stores.files` | `content` | `GET` | `/vector_stores/{vector_store_id}/files/{file_id}/content` | `missing` | `yes` |
| `videos` | `create` | `POST` | `/videos` | `missing` | `yes` |
| `videos` | `create_and_poll` | `-` | `(helper/no-http)` | `missing` | `no` |
| `videos` | `poll` | `-` | `(helper/no-http)` | `missing` | `no` |
| `videos` | `retrieve` | `GET` | `/videos/{video_id}` | `missing` | `yes` |
| `videos` | `list` | `GET` | `/videos` | `missing` | `yes` |
| `videos` | `delete` | `DELETE` | `/videos/{video_id}` | `missing` | `yes` |
| `videos` | `create_character` | `POST` | `/videos/characters` | `missing` | `yes` |
| `videos` | `download_content` | `GET` | `/videos/{video_id}/content` | `missing` | `yes` |
| `videos` | `edit` | `POST` | `/videos/edits` | `missing` | `yes` |
| `videos` | `extend` | `POST` | `/videos/extensions` | `missing` | `yes` |
| `videos` | `get_character` | `GET` | `/videos/characters/{character_id}` | `missing` | `yes` |
| `videos` | `remix` | `POST` | `/videos/{video_id}/remix` | `missing` | `yes` |
| `webhooks` | `unwrap` | `-` | `(helper/no-http)` | `missing` | `no` |
| `webhooks` | `verify_signature` | `-` | `(helper/no-http)` | `verify_signature` | `no` |
