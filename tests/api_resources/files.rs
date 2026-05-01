use crate::common::{file_object, request_capture_server, request_path_sequence_server, test_client, test_client_with_base_url};
use bytes::Bytes;
use futures_util::StreamExt;
use httpmock::prelude::*;
use ai_provider_sdk::{CursorPage, FileCreateParams, FileListParams, FileObject, FilePurpose, ListOrder, UploadFile};
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn files_create_sends_multipart_upload() {
    let (base_url, request_seen) = request_capture_server(
        "HTTP/1.1 200 OK",
        "{\"id\":\"file_123\",\"object\":\"file\",\"filename\":\"train.jsonl\",\"purpose\":\"fine-tune\"}",
    )
    .await;

    let client = test_client_with_base_url(base_url);
    let file = client
        .files()
        .create(FileCreateParams::new(
            UploadFile::from_bytes(
                "train.jsonl",
                Bytes::from_static(br#"{"prompt":"hi","completion":"there"}"#),
            ),
            FilePurpose::FineTune,
        ))
        .await
        .unwrap();

    let request = request_seen.lock().unwrap().clone().unwrap();
    let lower = request.to_ascii_lowercase();
    assert!(request.starts_with("POST /files?api-version=test HTTP/1.1"));
    assert!(lower.contains("content-type: multipart/form-data; boundary="));
    assert!(request.contains("name=\"purpose\""));
    assert!(request.contains("fine-tune"));
    assert!(request.contains("name=\"file\"; filename=\"train.jsonl\""));
    assert!(request.contains(r#"{"prompt":"hi","completion":"there"}"#));
    assert_eq!(file.id, "file_123");
}

#[tokio::test]
async fn files_list_returns_cursor_page() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/files")
            .query_param("limit", "2")
            .query_param("order", "desc")
            .query_param("purpose", "fine-tune");
        then.status(200).json_body(json!({
            "object": "list",
            "data": [
                {"id": "file_1", "object": "file", "filename": "a.jsonl", "purpose": "fine-tune"},
                {"id": "file_2", "object": "file", "filename": "b.jsonl", "purpose": "fine-tune"}
            ],
            "has_more": true
        }));
    });

    let client = test_client(&server);
    let mut params = FileListParams::new();
    params.limit = Some(2);
    params.order = Some(ListOrder::Desc);
    params.purpose = Some("fine-tune".to_string());

    let page = client.files().list_with_params(params).await.unwrap();

    mock.assert();
    assert!(page.has_next_page());
    assert_eq!(page.next_after(), Some("file_2"));
    assert_eq!(page.items().len(), 2);
}

#[tokio::test]
async fn files_list_next_page_uses_last_item_cursor() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/files")
            .query_param("after", "file_2")
            .query_param("limit", "2");
        then.status(200).json_body(json!({
            "object": "list",
            "data": [
                {"id": "file_3", "object": "file", "filename": "c.jsonl", "purpose": "fine-tune"}
            ],
            "has_more": false
        }));
    });

    let client = test_client(&server);
    let mut params = FileListParams::new();
    params.limit = Some(2);
    let current_page = CursorPage {
        object: Some("list".to_string()),
        data: vec![file_object("file_1"), file_object("file_2")],
        has_more: Some(true),
        extra: HashMap::new(),
    };

    let next_page = client
        .files()
        .list_next_page(&current_page, params)
        .await
        .unwrap()
        .unwrap();

    mock.assert();
    assert_eq!(next_page.items()[0].id, "file_3");
    assert!(!next_page.has_next_page());
}

#[tokio::test]
async fn files_list_auto_paging_streams_items_across_pages() {
    let (base_url, paths_seen) = request_path_sequence_server(vec![
        json!({
            "object": "list",
            "data": [
                {"id": "file_1", "object": "file", "filename": "a.jsonl", "purpose": "fine-tune"},
                {"id": "file_2", "object": "file", "filename": "b.jsonl", "purpose": "fine-tune"}
            ],
            "has_more": true
        })
        .to_string(),
        json!({
            "object": "list",
            "data": [
                {"id": "file_3", "object": "file", "filename": "c.jsonl", "purpose": "fine-tune"}
            ],
            "has_more": false
        })
        .to_string(),
    ])
    .await;

    let client = test_client_with_base_url(base_url);
    let mut params = FileListParams::new();
    params.limit = Some(2);

    let items: Vec<FileObject> = client
        .files()
        .list_auto_paging(params)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap();
    let paths = paths_seen.lock().unwrap();

    assert_eq!(
        items.iter().map(|file| file.id.as_str()).collect::<Vec<_>>(),
        vec!["file_1", "file_2", "file_3"]
    );
    assert_eq!(paths.len(), 2);
    assert!(paths[0].starts_with("/files?"));
    assert!(!paths[0].contains("after="));
    assert!(paths[0].contains("limit=2"));
    assert!(paths[1].starts_with("/files?"));
    assert!(paths[1].contains("after=file_2"));
    assert!(paths[1].contains("limit=2"));
}

#[tokio::test]
async fn files_content_returns_binary_response() {
    let (base_url, request_seen) = request_capture_server("HTTP/1.1 200 OK", "raw file bytes").await;
    let client = test_client_with_base_url(base_url);

    let bytes = client.files().content("file/123").await.unwrap();

    let request = request_seen.lock().unwrap().clone().unwrap();
    assert!(request.starts_with("GET /files/file%2F123/content?api-version=test HTTP/1.1"));
    assert!(request.to_ascii_lowercase().contains("accept: application/binary"));
    assert_eq!(bytes, Bytes::from_static(b"raw file bytes"));
}

#[tokio::test]
async fn files_retrieve_and_delete_use_file_id() {
    let server = MockServer::start();
    let retrieve = server.mock(|when, then| {
        when.method(GET).path("/files/file_123");
        then.status(200).json_body(json!({
            "id": "file_123",
            "object": "file",
            "filename": "train.jsonl",
            "purpose": "fine-tune"
        }));
    });
    let delete = server.mock(|when, then| {
        when.method(DELETE).path("/files/file_123");
        then.status(200).json_body(json!({
            "id": "file_123",
            "object": "file",
            "deleted": true
        }));
    });

    let client = test_client(&server);
    let file = client.files().retrieve("file_123").await.unwrap();
    let deleted = client.files().delete("file_123").await.unwrap();

    retrieve.assert();
    delete.assert();
    assert_eq!(file.id, "file_123");
    assert!(deleted.deleted);
}
