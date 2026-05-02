//! 模型序列化/反序列化测试。
//!
//! 对照 openai-python/tests/test_models.py，验证 Rust SDK 中
//! 所有数据模型的 serde 行为：基本字段、嵌套结构、可选字段、
//! 联合类型（untagged / tagged enum）、未知字段捕获、别名等。

use std::collections::HashMap;

use ai_provider_sdk::*;
use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// 1. 基本结构体反序列化（对应 Python test_basic）
// ---------------------------------------------------------------------------

#[test]
fn model_deserialize_basic_fields() {
    let raw = json!({
        "id": "gpt-4.1-mini",
        "object": "model",
        "created": 1720000000,
        "owned_by": "openai"
    });

    let m: Model = serde_json::from_value(raw).unwrap();
    assert_eq!(m.id, "gpt-4.1-mini");
    assert_eq!(m.object.as_deref(), Some("model"));
    assert_eq!(m.created, Some(1720000000));
    assert_eq!(m.owned_by.as_deref(), Some("openai"));
}

#[test]
fn model_serialize_round_trip() {
    let raw = json!({
        "id": "gpt-4.1-mini",
        "object": "model",
        "created": 1720000000,
        "owned_by": "openai"
    });

    let m: Model = serde_json::from_value(raw.clone()).unwrap();
    let out = serde_json::to_value(&m).unwrap();
    assert_eq!(out, raw);
}

// ---------------------------------------------------------------------------
// 2. 可选字段与默认值（对应 Python test_omitted_fields）
// ---------------------------------------------------------------------------

#[test]
fn optional_fields_absent() {
    let raw = json!({"id": "gpt-4.1-mini"});
    let m: Model = serde_json::from_value(raw).unwrap();
    assert_eq!(m.id, "gpt-4.1-mini");
    assert_eq!(m.object, None);
    assert_eq!(m.created, None);
    assert_eq!(m.owned_by, None);
}

#[test]
fn optional_fields_explicit_null() {
    let raw = json!({
        "id": "gpt-4.1-mini",
        "object": null,
        "created": null,
        "owned_by": null
    });
    let m: Model = serde_json::from_value(raw).unwrap();
    assert_eq!(m.id, "gpt-4.1-mini");
    assert_eq!(m.object, None);
    assert_eq!(m.created, None);
    assert_eq!(m.owned_by, None);
}

// ---------------------------------------------------------------------------
// 3. 嵌套模型（对应 Python test_directly_nested_model）
// ---------------------------------------------------------------------------

#[test]
fn nested_model_list() {
    let raw = json!({
        "object": "list",
        "data": [
            {"id": "gpt-4.1-mini", "object": "model"},
            {"id": "text-embedding-3-small", "object": "model", "owned_by": "openai"}
        ]
    });

    let list: ModelList = serde_json::from_value(raw).unwrap();
    assert_eq!(list.data.len(), 2);
    assert_eq!(list.data[0].id, "gpt-4.1-mini");
    assert_eq!(list.data[1].id, "text-embedding-3-small");
    assert_eq!(list.data[1].owned_by.as_deref(), Some("openai"));
}

// ---------------------------------------------------------------------------
// 4. 未知字段捕获（对应 Python test_unknown_fields）
// ---------------------------------------------------------------------------

#[test]
fn unknown_fields_captured_in_extra() {
    let raw = json!({
        "id": "chatcmpl-abc123",
        "choices": [{"index": 0, "message": {"role": "assistant", "content": "hi"}}],
        "usage": {"prompt_tokens": 5, "completion_tokens": 1, "total_tokens": 6},
        "new_feature": true,
        "nested_unknown": {"foo": "bar"}
    });

    let m: ChatCompletion = serde_json::from_value(raw).unwrap();
    assert_eq!(m.id, "chatcmpl-abc123");
    assert!(m.extra.contains_key("choices"));
    assert!(m.extra.contains_key("usage"));
    assert_eq!(m.extra["new_feature"], true);
    assert_eq!(m.extra["nested_unknown"]["foo"], "bar");
}

#[test]
fn unknown_fields_preserve_in_round_trip() {
    let raw = json!({
        "id": "chatcmpl-abc123",
        "future_field": [1, 2, 3]
    });

    let m: ChatCompletion = serde_json::from_value(raw.clone()).unwrap();
    let out = serde_json::to_value(&m).unwrap();
    assert_eq!(out, raw);
}

// ---------------------------------------------------------------------------
// 5. 枚举序列化（对应 Python test_aliases / enum rename）
// ---------------------------------------------------------------------------

#[test]
fn chat_role_lowercase_serialization() {
    assert_eq!(
        serde_json::to_string(&ChatRole::System).unwrap(),
        r#""system""#
    );
    assert_eq!(
        serde_json::to_string(&ChatRole::Developer).unwrap(),
        r#""developer""#
    );
    assert_eq!(
        serde_json::to_string(&ChatRole::User).unwrap(),
        r#""user""#
    );
    assert_eq!(
        serde_json::to_string(&ChatRole::Assistant).unwrap(),
        r#""assistant""#
    );
    assert_eq!(
        serde_json::to_string(&ChatRole::Tool).unwrap(),
        r#""tool""#
    );
}

#[test]
fn chat_role_deserialization() {
    let role: ChatRole = serde_json::from_str(r#""system""#).unwrap();
    assert_eq!(role, ChatRole::System);

    let role: ChatRole = serde_json::from_str(r#""user""#).unwrap();
    assert_eq!(role, ChatRole::User);
}

#[test]
fn file_purpose_rename() {
    assert_eq!(
        serde_json::to_string(&FilePurpose::FineTune).unwrap(),
        r#""fine-tune""#
    );
    assert_eq!(
        serde_json::to_string(&FilePurpose::FineTuneResults).unwrap(),
        r#""fine-tune-results""#
    );
    assert_eq!(
        serde_json::to_string(&FilePurpose::Assistants).unwrap(),
        r#""assistants""#
    );
}

#[test]
fn file_purpose_round_trip() {
    let purposes = [
        FilePurpose::Assistants,
        FilePurpose::AssistantsOutput,
        FilePurpose::Batch,
        FilePurpose::BatchOutput,
        FilePurpose::FineTune,
        FilePurpose::FineTuneResults,
        FilePurpose::Vision,
        FilePurpose::UserData,
        FilePurpose::Evals,
    ];
    for p in &purposes {
        let s = serde_json::to_string(p).unwrap();
        let back: FilePurpose = serde_json::from_str(&s).unwrap();
        assert_eq!(*p, back);
    }
}

#[test]
fn encoding_format_lowercase() {
    assert_eq!(
        serde_json::to_string(&EncodingFormat::Float).unwrap(),
        r#""float""#
    );
    assert_eq!(
        serde_json::to_string(&EncodingFormat::Base64).unwrap(),
        r#""base64""#
    );
}

// ---------------------------------------------------------------------------
// 6. Untagged 联合枚举（对应 Python test_nested_union_of_mixed_types）
// ---------------------------------------------------------------------------

#[test]
fn embedding_input_text_serialization() {
    let input = EmbeddingInput::Text("hello".to_string());
    let out = serde_json::to_string(&input).unwrap();
    assert_eq!(out, r#""hello""#);
}

#[test]
fn embedding_input_texts_serialization() {
    let input = EmbeddingInput::Texts(vec!["hello".to_string(), "world".to_string()]);
    let out = serde_json::to_string(&input).unwrap();
    assert_eq!(out, r#"["hello","world"]"#);
}

#[test]
fn embedding_input_tokens_serialization() {
    let input = EmbeddingInput::Tokens(vec![1, 2, 3]);
    let out = serde_json::to_string(&input).unwrap();
    assert_eq!(out, "[1,2,3]");
}

#[test]
fn embedding_input_token_batches_serialization() {
    let input = EmbeddingInput::TokenBatches(vec![vec![1, 2], vec![3, 4]]);
    let out = serde_json::to_string(&input).unwrap();
    assert_eq!(out, "[[1,2],[3,4]]");
}

#[test]
fn embedding_input_from_conversions() {
    let input: EmbeddingInput = "hello".into();
    assert_eq!(input, EmbeddingInput::Text("hello".to_string()));

    let input: EmbeddingInput = "hello".to_string().into();
    assert_eq!(input, EmbeddingInput::Text("hello".to_string()));

    let input: EmbeddingInput = vec!["a".to_string(), "b".to_string()].into();
    assert_eq!(
        input,
        EmbeddingInput::Texts(vec!["a".to_string(), "b".to_string()])
    );

    let input: EmbeddingInput = vec![1u32, 2].into();
    assert_eq!(input, EmbeddingInput::Tokens(vec![1, 2]));

    let input: EmbeddingInput = vec![vec![1u32], vec![2]].into();
    assert_eq!(input, EmbeddingInput::TokenBatches(vec![vec![1], vec![2]]));
}

#[test]
fn embedding_vector_float() {
    let vec: EmbeddingVector = serde_json::from_str(r#"[0.1, 0.2, 0.3]"#).unwrap();
    assert_eq!(vec, EmbeddingVector::Float(vec![0.1, 0.2, 0.3]));
}

#[test]
fn embedding_vector_base64() {
    let vec: EmbeddingVector = serde_json::from_str(r#""AQID""#).unwrap();
    assert_eq!(vec, EmbeddingVector::Base64("AQID".to_string()));
}

// ---------------------------------------------------------------------------
// 7. Discriminated / Tagged 联合枚举（对应 Python test_discriminated_unions）
// ---------------------------------------------------------------------------

#[test]
fn moderation_input_item_text_serialization() {
    let item = ModerationInputItem::text("Hello world");
    let out = serde_json::to_value(&item).unwrap();
    assert_eq!(out["type"], "text");
    assert_eq!(out["text"], "Hello world");
}

#[test]
fn moderation_input_item_image_url_serialization() {
    let item = ModerationInputItem::image_url("https://example.com/img.png");
    let out = serde_json::to_value(&item).unwrap();
    assert_eq!(out["type"], "image_url");
    assert_eq!(out["image_url"]["url"], "https://example.com/img.png");
}

#[test]
fn moderation_input_item_tagged_format() {
    let items = vec![
        ModerationInputItem::text("Hello"),
        ModerationInputItem::image_url("https://example.com/img.png"),
    ];
    let json = serde_json::to_string(&items).unwrap();
    // Verify tagged enum format: {"type": "text", ...}
    assert!(json.contains(r#""type":"text""#));
    assert!(json.contains(r#""type":"image_url""#));
}

// ---------------------------------------------------------------------------
// 8. 嵌套结构体的完整响应反序列化（对应 Python test_list_of_unions 等）
// ---------------------------------------------------------------------------

#[test]
fn create_embedding_response_full() {
    let raw = json!({
        "object": "list",
        "data": [
            {
                "object": "embedding",
                "index": 0,
                "embedding": [0.1, 0.2, 0.3]
            }
        ],
        "model": "text-embedding-3-small",
        "usage": {
            "prompt_tokens": 5,
            "total_tokens": 5
        }
    });

    let resp: CreateEmbeddingResponse = serde_json::from_value(raw).unwrap();
    assert_eq!(resp.object.as_deref(), Some("list"));
    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.data[0].index, 0);
    assert_eq!(
        resp.data[0].embedding,
        EmbeddingVector::Float(vec![0.1, 0.2, 0.3])
    );
    assert_eq!(resp.model.as_deref(), Some("text-embedding-3-small"));
    assert!(resp.usage.is_some());
    assert_eq!(resp.usage.as_ref().unwrap().prompt_tokens, 5);
}

#[test]
fn moderation_response_full() {
    let raw = json!({
        "id": "modr-abc123",
        "model": "omni-moderation-latest",
        "results": [{
            "flagged": true,
            "categories": {
                "violence": true,
                "sexual": false
            },
            "category_scores": {
                "violence": 0.95,
                "sexual": 0.01
            },
            "category_applied_input_types": {
                "violence": ["text"],
                "sexual": ["text", "image"]
            }
        }]
    });

    let resp: CreateModerationResponse = serde_json::from_value(raw).unwrap();
    assert_eq!(resp.id, "modr-abc123");
    assert_eq!(resp.results.len(), 1);
    assert!(resp.results[0].flagged);
    assert_eq!(resp.results[0].categories.violence, Some(true));
    assert_eq!(resp.results[0].categories.sexual, Some(false));
    assert_eq!(resp.results[0].category_scores.violence, Some(0.95));
    assert!(resp.results[0].category_applied_input_types.is_some());

    let applied = resp.results[0]
        .category_applied_input_types
        .as_ref()
        .unwrap();
    assert_eq!(applied.violence, Some(vec![ModerationAppliedInputType::Text]));
    assert_eq!(
        applied.sexual,
        Some(vec![
            ModerationAppliedInputType::Text,
            ModerationAppliedInputType::Image,
        ])
    );
}

// ---------------------------------------------------------------------------
// 9. 字段别名（对应 Python test_aliases / serde rename）
// ---------------------------------------------------------------------------

#[test]
fn moderation_categories_rename_fields() {
    let raw = json!({
        "self-harm": true,
        "sexual/minors": true,
        "hate/threatening": false,
        "violence/graphic": false,
        "self-harm/intent": true,
        "self-harm/instructions": false,
        "harassment/threatening": true,
        "illicit/violent": false
    });

    let cats: ModerationCategories = serde_json::from_value(raw).unwrap();
    assert_eq!(cats.self_harm, Some(true));
    assert_eq!(cats.sexual_minors, Some(true));
    assert_eq!(cats.hate_threatening, Some(false));
    assert_eq!(cats.violence_graphic, Some(false));
    assert_eq!(cats.self_harm_intent, Some(true));
    assert_eq!(cats.self_harm_instructions, Some(false));
    assert_eq!(cats.harassment_threatening, Some(true));
    assert_eq!(cats.illicit_violent, Some(false));
}

#[test]
fn moderation_categories_serialize_with_rename() {
    let mut cats = ModerationCategories::default();
    cats.self_harm = Some(true);
    cats.sexual_minors = Some(false);

    let out = serde_json::to_value(&cats).unwrap();
    assert_eq!(out["self-harm"], true);
    assert_eq!(out["sexual/minors"], false);
}

// ---------------------------------------------------------------------------
// 10. skip_serializing_if（对应 Python test_forwards_compat_model_dump_method）
// ---------------------------------------------------------------------------

#[test]
fn chat_completion_create_params_skips_none() {
    let params = ChatCompletionCreateParams::new("gpt-4.1-mini", vec![ChatMessage::user("hi")]);
    let out = serde_json::to_value(&params).unwrap();

    assert_eq!(out["model"], "gpt-4.1-mini");
    assert_eq!(out["messages"][0]["role"], "user");
    assert_eq!(out["messages"][0]["content"], "hi");
    // None fields should be absent
    assert!(out.get("temperature").is_none());
    assert!(out.get("top_p").is_none());
    assert!(out.get("max_completion_tokens").is_none());
    assert!(out.get("max_tokens").is_none());
    assert!(out.get("stream_options").is_none());
    assert!(out.get("store").is_none());
}

#[test]
fn chat_completion_create_params_includes_set_fields() {
    let mut params =
        ChatCompletionCreateParams::new("gpt-4.1-mini", vec![ChatMessage::user("hi")]);
    params.temperature = Some(0.7);
    params.max_completion_tokens = Some(100);
    params.store = Some(true);

    let out = serde_json::to_value(&params).unwrap();
    assert_eq!(out["temperature"], 0.7);
    assert_eq!(out["max_completion_tokens"], 100);
    assert_eq!(out["store"], true);
}

// ---------------------------------------------------------------------------
// 11. extra 扩展字段合并（对应 Python test_unknown_fields + extra_properties）
// ---------------------------------------------------------------------------

#[test]
fn chat_completion_create_params_extra_fields() {
    let mut params =
        ChatCompletionCreateParams::new("gpt-4.1-mini", vec![ChatMessage::user("hi")]);
    params.extra.insert("tools".to_string(), json!([{"type": "function"}]));
    params.extra.insert("seed".to_string(), json!(42));

    let out = serde_json::to_value(&params).unwrap();
    assert_eq!(out["tools"], json!([{"type": "function"}]));
    assert_eq!(out["seed"], 42);
}

#[test]
fn chat_message_extra_fields() {
    let raw = json!({
        "role": "assistant",
        "content": "Hello!",
        "name": "bot",
        "tool_calls": [{"id": "call_1", "type": "function", "function": {"name": "get_weather", "arguments": "{}"}}]
    });

    let msg: ChatMessage = serde_json::from_value(raw).unwrap();
    assert_eq!(msg.role, ChatRole::Assistant);
    assert_eq!(msg.content, "Hello!");
    assert_eq!(msg.extra["name"], "bot");
    assert!(msg.extra.contains_key("tool_calls"));
}

// ---------------------------------------------------------------------------
// 12. Debug trait（对应 Python test_repr）
// ---------------------------------------------------------------------------

#[test]
fn debug_format_basic() {
    let m = Model {
        id: "gpt-4.1-mini".to_string(),
        object: Some("model".to_string()),
        created: Some(1720000000),
        owned_by: Some("openai".to_string()),
        extra: HashMap::new(),
    };
    let debug_str = format!("{:?}", m);
    assert!(debug_str.contains("gpt-4.1-mini"));
    assert!(debug_str.contains("model"));
}

#[test]
fn debug_format_nested() {
    let list = ModelList {
        object: Some("list".to_string()),
        data: vec![Model {
            id: "gpt-4.1-mini".to_string(),
            object: Some("model".to_string()),
            created: None,
            owned_by: None,
            extra: HashMap::new(),
        }],
        extra: HashMap::new(),
    };
    let debug_str = format!("{:?}", list);
    assert!(debug_str.contains("ModelList"));
    assert!(debug_str.contains("gpt-4.1-mini"));
}

// ---------------------------------------------------------------------------
// 13. 游标分页（对应 Python test_optional_list + 分页逻辑）
// ---------------------------------------------------------------------------

#[test]
fn cursor_page_deserialize() {
    let raw = json!({
        "object": "list",
        "data": [
            {"id": "chatcmpl-1"},
            {"id": "chatcmpl-2"}
        ],
        "has_more": true
    });

    let page: CursorPage<ChatCompletion> = serde_json::from_value(raw).unwrap();
    assert_eq!(page.data.len(), 2);
    assert_eq!(page.has_more, Some(true));
    assert!(page.has_next_page());
    assert_eq!(page.next_after(), Some("chatcmpl-2"));
}

#[test]
fn cursor_page_no_more() {
    let raw = json!({
        "object": "list",
        "data": [{"id": "chatcmpl-1"}],
        "has_more": false
    });

    let page: CursorPage<ChatCompletion> = serde_json::from_value(raw).unwrap();
    assert!(!page.has_next_page());
}

#[test]
fn cursor_page_empty_data() {
    let raw = json!({
        "object": "list",
        "data": [],
        "has_more": false
    });

    let page: CursorPage<ChatCompletion> = serde_json::from_value(raw).unwrap();
    assert!(page.data.is_empty());
    assert!(!page.has_next_page());
    assert_eq!(page.next_after(), None);
}

#[test]
fn cursor_page_items_and_into_items() {
    let raw = json!({
        "data": [{"id": "a"}, {"id": "b"}],
        "has_more": true
    });

    let page: CursorPage<ChatCompletion> = serde_json::from_value(raw).unwrap();
    assert_eq!(page.items().len(), 2);
    let items = page.into_items();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].id, "a");
    assert_eq!(items[1].id, "b");
}

// ---------------------------------------------------------------------------
// 14. 完整文件对象（对应 Python 多层嵌套测试）
// ---------------------------------------------------------------------------

#[test]
fn file_object_all_optional_fields() {
    let raw = json!({
        "id": "file-abc123",
        "bytes": 120000,
        "created_at": 1720000000,
        "filename": "train.jsonl",
        "object": "file",
        "purpose": "fine-tune",
        "status": "processed",
        "expires_at": null,
        "status_details": null
    });

    let f: FileObject = serde_json::from_value(raw).unwrap();
    assert_eq!(f.id, "file-abc123");
    assert_eq!(f.bytes, Some(120000));
    assert_eq!(f.created_at, Some(1720000000));
    assert_eq!(f.filename.as_deref(), Some("train.jsonl"));
    assert_eq!(f.object.as_deref(), Some("file"));
    assert_eq!(f.purpose, Some(FilePurpose::FineTune));
    assert_eq!(f.status.as_deref(), Some("processed"));
    assert_eq!(f.expires_at, None);
    assert_eq!(f.status_details, None);
}

#[test]
fn file_deleted() {
    let raw = json!({
        "id": "file-abc123",
        "deleted": true,
        "object": "file"
    });

    let d: FileDeleted = serde_json::from_value(raw).unwrap();
    assert_eq!(d.id, "file-abc123");
    assert!(d.deleted);
    assert_eq!(d.object.as_deref(), Some("file"));
}

// ---------------------------------------------------------------------------
// 15. 序列化请求参数（对应 Python test_to_dict / test_to_json）
// ---------------------------------------------------------------------------

#[test]
fn embedding_create_params_serialization() {
    let params = EmbeddingCreateParams::new("text-embedding-3-small", "hello");
    let out = serde_json::to_value(&params).unwrap();
    assert_eq!(out["model"], "text-embedding-3-small");
    assert_eq!(out["input"], "hello");
    assert!(out.get("dimensions").is_none());
    assert!(out.get("encoding_format").is_none());
}

#[test]
fn moderation_create_params_text_input() {
    let params = ModerationCreateParams::new("check this text").model("omni-moderation-latest");
    let out = serde_json::to_value(&params).unwrap();
    assert_eq!(out["input"], "check this text");
    assert_eq!(out["model"], "omni-moderation-latest");
}

#[test]
fn moderation_create_params_multimodal_input() {
    let params = ModerationCreateParams::new(vec![
        ModerationInputItem::text("Hello"),
        ModerationInputItem::image_url("https://example.com/img.png"),
    ]);
    let out = serde_json::to_value(&params).unwrap();
    assert_eq!(out["input"][0]["type"], "text");
    assert_eq!(out["input"][0]["text"], "Hello");
    assert_eq!(out["input"][1]["type"], "image_url");
}

// ---------------------------------------------------------------------------
// 16. 错误处理（对应 Python 反序列化不匹配测试）
// ---------------------------------------------------------------------------

#[test]
fn missing_required_field_fails() {
    let raw = json!({"object": "model"});
    let result = serde_json::from_value::<Model>(raw);
    assert!(result.is_err());
}

#[test]
fn invalid_enum_variant_fails() {
    let raw = json!({"role": "invalid_role", "content": "hi"});
    let result = serde_json::from_value::<ChatMessage>(raw);
    assert!(result.is_err());
}

#[test]
fn wrong_type_for_field_fails() {
    let raw = json!({"id": 12345});
    let result = serde_json::from_value::<Model>(raw);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 17. ChatCompletionStoreMessage 可选字段组合（对应 Python 多可选测试）
// ---------------------------------------------------------------------------

#[test]
fn store_message_minimal() {
    let raw = json!({"id": "msg-abc123"});
    let msg: ChatCompletionStoreMessage = serde_json::from_value(raw).unwrap();
    assert_eq!(msg.id, "msg-abc123");
    assert_eq!(msg.role, None);
    assert_eq!(msg.content, None);
    assert_eq!(msg.object, None);
    assert_eq!(msg.created_at, None);
}

#[test]
fn store_message_full() {
    let raw = json!({
        "id": "msg-abc123",
        "role": "assistant",
        "content": "Here is the answer",
        "object": "chat.completion.message",
        "created_at": 1720000000,
        "metadata": {"source": "api"}
    });

    let msg: ChatCompletionStoreMessage = serde_json::from_value(raw).unwrap();
    assert_eq!(msg.id, "msg-abc123");
    assert_eq!(msg.role, Some(ChatRole::Assistant));
    assert_eq!(msg.content, Some(Value::String("Here is the answer".to_string())));
    assert_eq!(msg.object.as_deref(), Some("chat.completion.message"));
    assert_eq!(msg.created_at, Some(1720000000));
    assert_eq!(msg.extra["metadata"]["source"], "api");
}

// ---------------------------------------------------------------------------
// 18. ChatCompletionDeleted（对应 Python 简单结构体）
// ---------------------------------------------------------------------------

#[test]
fn chat_completion_deleted() {
    let raw = json!({
        "id": "chatcmpl-abc123",
        "deleted": true,
        "object": "chat.completion.deleted"
    });

    let d: ChatCompletionDeleted = serde_json::from_value(raw).unwrap();
    assert_eq!(d.id, "chatcmpl-abc123");
    assert!(d.deleted);
    assert_eq!(d.object.as_deref(), Some("chat.completion.deleted"));
}

// ---------------------------------------------------------------------------
// 19. EmbeddingUsage（对应 Python 简单子结构）
// ---------------------------------------------------------------------------

#[test]
fn embedding_usage() {
    let raw = json!({
        "prompt_tokens": 10,
        "total_tokens": 10
    });

    let usage: EmbeddingUsage = serde_json::from_value(raw).unwrap();
    assert_eq!(usage.prompt_tokens, 10);
    assert_eq!(usage.total_tokens, 10);
}

// ---------------------------------------------------------------------------
// 20. UploadFile + FileCreateParams（对应 Python 构造器测试）
// ---------------------------------------------------------------------------

#[test]
fn upload_file_from_bytes() {
    use bytes::Bytes;

    let f = UploadFile::from_bytes("train.jsonl", Bytes::from_static(b"line1\nline2\n"));
    assert_eq!(f.file_name, "train.jsonl");
    assert_eq!(&f.bytes[..], b"line1\nline2\n");
    assert_eq!(f.mime_type, None);
}

#[test]
fn file_create_params_constructor() {
    use bytes::Bytes;

    let file = UploadFile::from_bytes("data.jsonl", Bytes::new());
    let params = FileCreateParams::new(file, FilePurpose::Batch);
    assert_eq!(params.file.file_name, "data.jsonl");
    assert_eq!(params.purpose, FilePurpose::Batch);
    assert_eq!(params.expires_after, None);
    assert!(params.extra.is_empty());
}

// ---------------------------------------------------------------------------
// 21. 列表参数序列化（对应 Python test_forwards_compat_model_dump_method）
// ---------------------------------------------------------------------------

#[test]
fn chat_completion_list_params_skips_none() {
    let params = ChatCompletionListParams::new();
    let out = serde_json::to_value(&params).unwrap();
    assert!(out.get("after").is_none());
    assert!(out.get("limit").is_none());
    assert!(out.get("metadata").is_none());
    assert!(out.get("model").is_none());
    assert!(out.get("order").is_none());
}

#[test]
fn chat_completion_list_params_with_values() {
    let mut params = ChatCompletionListParams::new();
    params.after = Some("chatcmpl-abc".to_string());
    params.limit = Some(10);
    params.model = Some("gpt-4.1-mini".to_string());
    params.order = Some(ChatListOrder::Desc);

    let out = serde_json::to_value(&params).unwrap();
    assert_eq!(out["after"], "chatcmpl-abc");
    assert_eq!(out["limit"], 10);
    assert_eq!(out["model"], "gpt-4.1-mini");
    assert_eq!(out["order"], "desc");
}

// ---------------------------------------------------------------------------
// 22. ChatMessage 快捷构造器（对应 Python 构造器测试）
// ---------------------------------------------------------------------------

#[test]
fn chat_message_user_constructor() {
    let msg = ChatMessage::user("Hello!");
    assert_eq!(msg.role, ChatRole::User);
    assert_eq!(msg.content, "Hello!");
    assert!(msg.extra.is_empty());
}

#[test]
fn chat_message_developer_constructor() {
    let msg = ChatMessage::developer("You are a helpful assistant");
    assert_eq!(msg.role, ChatRole::Developer);
    assert_eq!(msg.content, "You are a helpful assistant");
    assert!(msg.extra.is_empty());
}

// ---------------------------------------------------------------------------
// 23. ChatCompletionUpdateParams（对应 Python 简单参数）
// ---------------------------------------------------------------------------

#[test]
fn chat_completion_update_params() {
    let mut metadata = HashMap::new();
    metadata.insert("key".to_string(), "value".to_string());
    let params = ChatCompletionUpdateParams::new(metadata.clone());

    let out = serde_json::to_value(&params).unwrap();
    assert_eq!(out["metadata"]["key"], "value");
}

// ---------------------------------------------------------------------------
// 24. ExpiresAfter（对应 Python 嵌套子结构）
// ---------------------------------------------------------------------------

#[test]
fn expires_after_round_trip() {
    let ea = ExpiresAfter {
        anchor: "last_active_time".to_string(),
        seconds: 86400,
    };
    let json = serde_json::to_string(&ea).unwrap();
    let back: ExpiresAfter = serde_json::from_str(&json).unwrap();
    assert_eq!(back.anchor, "last_active_time");
    assert_eq!(back.seconds, 86400);
}

// ---------------------------------------------------------------------------
// 25. ModerationInput untagged enum（对应 Python test_union_of_lists）
// ---------------------------------------------------------------------------

#[test]
fn moderation_input_text_serialization() {
    let input = ModerationInput::Text("check this".to_string());
    let out = serde_json::to_string(&input).unwrap();
    assert_eq!(out, r#""check this""#);
}

#[test]
fn moderation_input_texts_serialization() {
    let input = ModerationInput::Texts(vec!["a".to_string(), "b".to_string()]);
    let out = serde_json::to_string(&input).unwrap();
    assert_eq!(out, r#"["a","b"]"#);
}

#[test]
fn moderation_input_items_serialization() {
    let input = ModerationInput::Items(vec![
        ModerationInputItem::text("hello"),
        ModerationInputItem::image_url("https://x.com/i.png"),
    ]);
    let out = serde_json::to_value(&input).unwrap();
    assert_eq!(out[0]["type"], "text");
    assert_eq!(out[1]["type"], "image_url");
}

#[test]
fn moderation_input_from_conversions() {
    let input: ModerationInput = "hello".into();
    assert_eq!(input, ModerationInput::Text("hello".to_string()));

    let input: ModerationInput = vec!["a".to_string()].into();
    assert_eq!(
        input,
        ModerationInput::Texts(vec!["a".to_string()])
    );
}

// ---------------------------------------------------------------------------
// 26. PartialEq 验证（对应 Python assert == 测试）
// ---------------------------------------------------------------------------

#[test]
fn model_equality() {
    let m1 = Model {
        id: "gpt-4.1-mini".to_string(),
        object: Some("model".to_string()),
        created: Some(1720000000),
        owned_by: Some("openai".to_string()),
        extra: HashMap::new(),
    };
    let m2 = m1.clone();
    assert_eq!(m1, m2);
}

#[test]
fn chat_role_equality() {
    assert_eq!(ChatRole::User, ChatRole::User);
    assert_ne!(ChatRole::User, ChatRole::Assistant);
}
