//! SSE（Server-Sent Events）流式解码层。
//!
//! 把 HTTP 字节流按 SSE 协议解析为结构化事件。
//! 主要用于 Chat Completions 和 Responses 的流式模式。
//!
//! ## SSE 协议简述
//!
//! SSE 是一种基于 HTTP 的单向文本协议，服务端逐行发送：
//!
//! ```text
//! event: message       ← 事件类型（可选）
//! data: {"delta":"hi"} ← 数据行（可多行）
//! id: evt_42           ← 事件 ID（用于断线续传）
//! retry: 3000          ← 建议重连间隔（毫秒）
//!                      ← 空行 = 事件分隔符
//! data: [DONE]         ← OpenAI 约定的流结束标记
//! ```
//!
//! ## 核心组件
//!
//! - [`SseStream`]：持有 HTTP 响应，调用 `events()` 返回异步事件流。
//! - [`SseDecoder`]：增量式状态机，处理分包、多行 `data:` 与 `\r\n` 换行。
//! - [`ServerSentEvent`]：解码后的结构化事件。

use async_stream::try_stream;
use bytes::{Buf, Bytes, BytesMut};
use futures_core::Stream;
use futures_util::StreamExt;
use serde_json::Value;

use crate::error::{Error, Result};

/// 解码后的 SSE 事件结构。
///
/// 所有字段均与 SSE 协议规范一一对应。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerSentEvent {
    /// SSE `event` 字段；为空时表示默认事件类型（`message`）。
    pub event: Option<String>,
    /// SSE `data` 字段拼接结果（多行 `data:` 以换行 `\n` 连接）。
    pub data: String,
    /// SSE `id` 字段；用于客户端断线续传语义（`Last-Event-ID`）。
    pub id: Option<String>,
    /// SSE `retry` 字段；服务端建议的重连间隔（毫秒）。
    pub retry: Option<u64>,
}

/// SSE 流包装器，持有原始 HTTP 响应。
///
/// 调用 [`SseStream::events()`] 消费响应体并返回解码后的异步事件流。
pub struct SseStream {
    response: reqwest::Response,
}

impl SseStream {
    /// 从 HTTP 响应创建 SSE 流包装器。
    ///
    /// 调用方应确保响应状态码已检查（通常在 `Transport` 层完成）。
    pub(crate) fn new(response: reqwest::Response) -> Self {
        Self { response }
    }

    /// 将 HTTP 响应体按 SSE 协议解码为事件流。
    ///
    /// ## 行为边界
    ///
    /// - 遇到 `data: [DONE]` 立即结束流（OpenAI 约定的流终止标记）。
    /// - 若 `data` 可解析为 JSON 且包含 `error` 字段，抛出 [`Error::Stream`]。
    /// - 流结束后自动冲刷解码器缓冲区中的剩余事件。
    ///
    /// ## 示例
    ///
    /// ```no_run
    /// use futures_util::StreamExt;
    /// use std::pin::pin;
    /// # async fn example(stream: ai_provider_sdk::SseStream) {
    /// let mut events = pin!(stream.events());
    /// while let Some(result) = events.next().await {
    ///     let event = result.unwrap();
    ///     println!("data: {}", event.data);
    /// }
    /// # }
    /// ```
    pub fn events(self) -> impl Stream<Item = Result<ServerSentEvent>> {
        let mut chunks = self.response.bytes_stream();

        try_stream! {
            let mut decoder = SseDecoder::new();
            // 逐 chunk 推入解码器，产出完整事件
            while let Some(chunk) = chunks.next().await {
                let chunk = chunk.map_err(|err| Error::Stream(err.to_string()))?;
                for event in decoder.push(chunk)? {
                    // [DONE] 是 OpenAI 约定的流结束标记
                    if event.data.starts_with("[DONE]") {
                        return;
                    }
                    // 检查流中嵌入的错误事件
                    if let Ok(data) = serde_json::from_str::<Value>(&event.data) {
                        if let Some(error) = data.get("error") {
                            Err(Error::Stream(
                                error
                                    .get("message")
                                    .and_then(Value::as_str)
                                    .unwrap_or("An error occurred during streaming")
                                    .to_string(),
                            ))?;
                        }
                    }
                    yield event;
                }
            }

            // 冲刷解码器中可能残留的最后一个不完整事件
            for event in decoder.finish()? {
                if event.data.starts_with("[DONE]") {
                    return;
                }
                yield event;
            }
        }
    }
}

#[derive(Debug, Default)]
/// SSE 增量式解码器状态机。
///
/// 状态字段在 `push` 过程中跨 chunk 保持，用于处理以下场景：
/// - **分包**：一个 SSE 事件可能跨越多个 TCP chunk 到达。
/// - **多行 `data:`**：同一事件的多个 `data:` 行需以 `\n` 拼接。
/// - **混合换行**：兼容 `\n` 和 `\r\n` 两种换行符。
///
/// ## 使用方式
///
/// ```text
/// let mut decoder = SseDecoder::new();
/// let events = decoder.push(chunk1)?;  // 可能返回 0 个或多个事件
/// let events = decoder.push(chunk2)?;  // 后续 chunk 可能补完之前的事件
/// let events = decoder.finish()?;      // 流结束时冲刷残留
/// ```
pub struct SseDecoder {
    /// 内部字节缓冲区，存放尚未组成完整行的数据。
    bytes: BytesMut,
    /// 当前事件的 `event` 字段（在遇到空行分隔符时重置）。
    event: Option<String>,
    /// 当前事件的 `data` 行集合（多行 `data:` 以换行连接）。
    data: Vec<String>,
    /// 最近一次 `id` 字段值（SSE 协议要求跨事件保留）。
    last_event_id: Option<String>,
    /// 最近一次 `retry` 字段值（SSE 协议要求跨事件保留）。
    retry: Option<u64>,
}

impl SseDecoder {
    /// 创建空状态解码器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 推入一个字节分片并返回当前可产出的完整事件集合。
    ///
    /// 该方法可重复调用；未组成完整行/事件的数据会保留在内部缓冲区。
    /// 每次调用可能返回 0 到 N 个事件。
    pub fn push(&mut self, chunk: Bytes) -> Result<Vec<ServerSentEvent>> {
        self.bytes.extend_from_slice(&chunk);
        let mut events = Vec::new();

        // 逐行解析，直到缓冲区中没有完整行
        while let Some(line) = self.next_line()? {
            if let Some(event) = self.decode_line(&line) {
                events.push(event);
            }
        }

        Ok(events)
    }

    /// 在底层流结束时冲刷缓冲区，产出剩余事件。
    ///
    /// 如果缓冲区中还有不完整的行（未以换行符结尾），会尝试作为最后一行解析。
    /// 同时会冲刷尚未通过空行分隔符触发的累积事件。
    pub fn finish(&mut self) -> Result<Vec<ServerSentEvent>> {
        let mut events = Vec::new();

        // 处理缓冲区中可能残留的不完整行（未以 \n 或 \r\n 结尾）
        if !self.bytes.is_empty() {
            let line = std::str::from_utf8(&self.bytes)
                .map_err(|err| Error::Stream(err.to_string()))?
                .to_string();
            self.bytes.clear();
            if let Some(event) = self.decode_line(&line) {
                events.push(event);
            }
        }

        // 冲刷尚未通过空行触发的事件
        if let Some(event) = self.flush_event() {
            events.push(event);
        }

        Ok(events)
    }

    /// 从内部缓冲区读取下一行（兼容 `\n` 和 `\r\n`）。
    ///
    /// 返回 `None` 表示缓冲区中没有完整的行（等待更多数据）。
    fn next_line(&mut self) -> Result<Option<String>> {
        // 查找第一个换行符位置
        let Some(pos) = self
            .bytes
            .iter()
            .position(|byte| *byte == b'\n' || *byte == b'\r')
        else {
            return Ok(None);
        };

        // 分割出行内容（不含换行符）
        let line = self.bytes.split_to(pos);
        let newline = self.bytes.get_u8();

        // 处理 \r\n：如果换行符是 \r，且下一个字节是 \n，则跳过
        if newline == b'\r' && self.bytes.first() == Some(&b'\n') {
            self.bytes.advance(1);
        }

        let line = std::str::from_utf8(&line)
            .map_err(|err| Error::Stream(err.to_string()))?
            .to_string();
        Ok(Some(line))
    }

    /// 解码单行 SSE 字段；仅在遇到事件分隔空行时返回完整事件。
    ///
    /// - 空行（`""`）→ 触发 `flush_event()` 封装并返回当前累积的事件。
    /// - 注释行（以 `:` 开头）→ 忽略（SSE 心跳 / 注释）。
    /// - `event:` 行 → 设置事件类型。
    /// - `data:` 行 → 追加到数据行集合。
    /// - `id:` 行 → 更新 `last_event_id`（SSE 规范：含 `\0` 的 id 被忽略）。
    /// - `retry:` 行 → 解析为毫秒整数。
    fn decode_line(&mut self, line: &str) -> Option<ServerSentEvent> {
        // 空行 = 事件分隔符，触发封装
        if line.is_empty() {
            return self.flush_event();
        }

        // 注释行（SSE 心跳），忽略
        if line.starts_with(':') {
            return None;
        }

        // 解析 "field: value" 或 "field"（无冒号时 value 为空）
        let (field, value) = line.split_once(':').unwrap_or((line, ""));
        // SSE 规范：冒号后第一个空格是分隔符的一部分，不作为值
        let value = value.strip_prefix(' ').unwrap_or(value);

        match field {
            "event" => self.event = Some(value.to_string()),
            "data" => self.data.push(value.to_string()),
            // SSE 规范：id 字段中包含空字符 \0 时忽略
            "id" if !value.contains('\0') => self.last_event_id = Some(value.to_string()),
            "retry" => self.retry = value.parse().ok(),
            _ => {} // 未知字段忽略
        }

        None
    }

    /// 将当前累积的字段封装为事件，并清理可重置状态。
    ///
    /// 注意：`last_event_id` 和 `retry` 按 SSE 规范需跨事件保留，
    /// 这里对 `retry` 做 `take()` 是因为每次封装后不需要再次传递。
    /// `last_event_id` 使用 `clone()` 保留在解码器中。
    fn flush_event(&mut self) -> Option<ServerSentEvent> {
        // 所有字段都为空时不产出事件
        if self.event.is_none()
            && self.data.is_empty()
            && self.last_event_id.is_none()
            && self.retry.is_none()
        {
            return None;
        }

        let event = ServerSentEvent {
            event: self.event.take(),
            // 多行 data 以换行连接（SSE 规范要求）
            data: self.data.join("\n"),
            id: self.last_event_id.clone(),
            retry: self.retry.take(),
        };
        // 清理 data 行，为下一个事件做准备
        self.data.clear();
        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_complete_event() {
        let mut decoder = SseDecoder::new();
        let events = decoder
            .push(Bytes::from_static(
                b"event: ping\ndata: {\"x\":1}\nid: abc\n\n",
            ))
            .unwrap();

        assert_eq!(
            events,
            vec![ServerSentEvent {
                event: Some("ping".to_string()),
                data: "{\"x\":1}".to_string(),
                id: Some("abc".to_string()),
                retry: None,
            }]
        );
    }

    #[test]
    fn decodes_split_event_and_multi_data_lines() {
        let mut decoder = SseDecoder::new();
        // 第一个 chunk 包含不完整事件（只有一行 data，没有空行分隔符）
        assert!(decoder
            .push(Bytes::from_static(b"data: a\n"))
            .unwrap()
            .is_empty());
        // 第二个 chunk 补完事件
        let events = decoder.push(Bytes::from_static(b"data: b\n\n")).unwrap();

        // 多行 data 以换行连接
        assert_eq!(events[0].data, "a\nb");
    }

    #[test]
    fn keeps_last_event_id_across_events() {
        let mut decoder = SseDecoder::new();
        let events = decoder
            .push(Bytes::from_static(b"id: one\ndata: a\n\ndata: b\n\n"))
            .unwrap();

        // SSE 规范：last_event_id 跨事件保留
        assert_eq!(events[0].id.as_deref(), Some("one"));
        assert_eq!(events[1].id.as_deref(), Some("one"));
    }
}
