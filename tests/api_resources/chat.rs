//! Chat Completions API 资源测试。
//!
//! 验证 Chat Completions 的同步和流式调用路径：
//! - 请求体构造（model、messages、stream 字段）
//! - 响应体反序列化（id、extra 字段）
//! - SSE 流式解码直到 `[DONE]`

use crate::common::test_client;
use ai_provider_sdk::{
    ChatCompletionCreateParams, ChatCompletionListParams, ChatCompletionMessageListParams,
    ChatCompletionUpdateParams, ChatListOrder, ChatMessage,
};
use futures_util::StreamExt;
use httpmock::prelude::*;
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
/// 验证 Chat Completions 同步调用发送了正确的请求并解析了响应。
async fn chat_completions_create_sends_expected_request() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/chat/completions")
            .json_body(json!({
                "model": "gpt-4.1-mini",
                "messages": [{"role": "user", "content": "hello"}],
                "stream": false
            }));
        then.status(200).json_body(json!({
            "id": "chatcmpl_123",
            "choices": []
        }));
    });

    let client = test_client(&server);
    let completion = client
        .chat()
        .completions()
        .create(ChatCompletionCreateParams::new(
            "gpt-4.1-mini",
            vec![ChatMessage::user("hello")],
        ))
        .await
        .unwrap();

    mock.assert();
    assert_eq!(completion.id, "chatcmpl_123");
    assert_eq!(completion.extra["choices"], json!([]));
}

#[tokio::test]
/// 验证 Chat Completions 流式调用正确解码 SSE 事件直到 `[DONE]`。
async fn streams_sse_events_until_done() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/chat/completions")
            .json_body(json!({
                "model": "gpt-4.1-mini",
                "messages": [{"role": "user", "content": "hello"}],
                "stream": true
            }));
        then.status(200)
            .header("content-type", "text/event-stream")
            .body("data: {\"id\":\"chunk_1\"}\n\ndata: [DONE]\n\n");
    });

    let client = test_client(&server);
    let stream = client
        .chat()
        .completions()
        .create_stream(ChatCompletionCreateParams::new(
            "gpt-4.1-mini",
            vec![ChatMessage::user("hello")],
        ))
        .await
        .unwrap();

    let events: Vec<_> = stream
        .events()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();

    mock.assert();
    // 应收到 1 个事件（[DONE] 不产出事件，只终止流）
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data, "{\"id\":\"chunk_1\"}");
}

#[tokio::test]
/// 验证 stored Chat Completion 查询、更新和删除使用正确的资源路径。
async fn chat_completions_stored_retrieve_update_delete_use_completion_id() {
    let server = MockServer::start();
    let retrieve = server.mock(|when, then| {
        when.method(GET).path("/chat/completions/chatcmpl_123");
        then.status(200).json_body(json!({
            "id": "chatcmpl_123",
            "object": "chat.completion",
            "metadata": {"env": "test"}
        }));
    });
    let update = server.mock(|when, then| {
        when.method(POST)
            .path("/chat/completions/chatcmpl_123")
            .json_body(json!({
                "metadata": {"env": "prod"}
            }));
        then.status(200).json_body(json!({
            "id": "chatcmpl_123",
            "object": "chat.completion",
            "metadata": {"env": "prod"}
        }));
    });
    let delete = server.mock(|when, then| {
        when.method(DELETE).path("/chat/completions/chatcmpl_123");
        then.status(200).json_body(json!({
            "id": "chatcmpl_123",
            "object": "chat.completion.deleted",
            "deleted": true
        }));
    });

    let client = test_client(&server);
    let completion = client
        .chat()
        .completions()
        .retrieve("chatcmpl_123")
        .await
        .unwrap();
    let updated = client
        .chat()
        .completions()
        .update(
            "chatcmpl_123",
            ChatCompletionUpdateParams::new(HashMap::from([(
                "env".to_string(),
                "prod".to_string(),
            )])),
        )
        .await
        .unwrap();
    let deleted = client
        .chat()
        .completions()
        .delete("chatcmpl_123")
        .await
        .unwrap();

    retrieve.assert();
    update.assert();
    delete.assert();
    assert_eq!(completion.id, "chatcmpl_123");
    assert_eq!(updated.extra["metadata"], json!({"env": "prod"}));
    assert!(deleted.deleted);
}

#[tokio::test]
/// 验证 stored Chat Completions 列表查询参数与分页响应解析。
async fn chat_completions_list_returns_cursor_page() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/chat/completions")
            .query_param("limit", "2")
            .query_param("order", "desc")
            .query_param("model", "gpt-4.1-mini")
            .query_param("metadata[env]", "test");
        then.status(200).json_body(json!({
            "object": "list",
            "data": [
                {"id": "chatcmpl_1", "object": "chat.completion"},
                {"id": "chatcmpl_2", "object": "chat.completion"}
            ],
            "has_more": true
        }));
    });

    let client = test_client(&server);
    let mut params = ChatCompletionListParams::new();
    params.limit = Some(2);
    params.order = Some(ChatListOrder::Desc);
    params.model = Some("gpt-4.1-mini".to_string());
    params.metadata = Some(HashMap::from([("env".to_string(), "test".to_string())]));

    let page = client
        .chat()
        .completions()
        .list_with_params(params)
        .await
        .unwrap();

    mock.assert();
    assert!(page.has_next_page());
    assert_eq!(page.next_after(), Some("chatcmpl_2"));
    assert_eq!(page.items().len(), 2);
}

#[tokio::test]
/// 验证 stored Chat Completion messages 子资源的列表路径和查询参数。
async fn chat_completion_messages_list_uses_nested_resource_path() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/chat/completions/chatcmpl_123/messages")
            .query_param("after", "msg_1")
            .query_param("limit", "1")
            .query_param("order", "asc");
        then.status(200).json_body(json!({
            "object": "list",
            "data": [
                {"id": "msg_2", "role": "assistant", "content": "hello"}
            ],
            "has_more": false
        }));
    });

    let client = test_client(&server);
    let mut params = ChatCompletionMessageListParams::new();
    params.after = Some("msg_1".to_string());
    params.limit = Some(1);
    params.order = Some(ChatListOrder::Asc);

    let page = client
        .chat()
        .completions()
        .messages()
        .list_with_params("chatcmpl_123", params)
        .await
        .unwrap();

    mock.assert();
    assert_eq!(page.items()[0].id, "msg_2");
    assert_eq!(
        page.items()[0].role.as_ref().unwrap(),
        &ai_provider_sdk::ChatRole::Assistant
    );
    assert!(!page.has_next_page());
}
