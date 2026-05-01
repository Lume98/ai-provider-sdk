use async_stream::try_stream;
use bytes::{Buf, Bytes, BytesMut};
use futures_core::Stream;
use futures_util::StreamExt;
use serde_json::Value;

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerSentEvent {
    pub event: Option<String>,
    pub data: String,
    pub id: Option<String>,
    pub retry: Option<u64>,
}

pub struct SseStream {
    response: reqwest::Response,
}

impl SseStream {
    pub(crate) fn new(response: reqwest::Response) -> Self {
        Self { response }
    }

    pub fn events(self) -> impl Stream<Item = Result<ServerSentEvent>> {
        let mut chunks = self.response.bytes_stream();

        try_stream! {
            let mut decoder = SseDecoder::new();
            while let Some(chunk) = chunks.next().await {
                let chunk = chunk.map_err(|err| Error::Stream(err.to_string()))?;
                for event in decoder.push(chunk)? {
                    if event.data.starts_with("[DONE]") {
                        return;
                    }
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
pub struct SseDecoder {
    bytes: BytesMut,
    event: Option<String>,
    data: Vec<String>,
    last_event_id: Option<String>,
    retry: Option<u64>,
}

impl SseDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, chunk: Bytes) -> Result<Vec<ServerSentEvent>> {
        self.bytes.extend_from_slice(&chunk);
        let mut events = Vec::new();

        while let Some(line) = self.next_line()? {
            if let Some(event) = self.decode_line(&line) {
                events.push(event);
            }
        }

        Ok(events)
    }

    pub fn finish(&mut self) -> Result<Vec<ServerSentEvent>> {
        let mut events = Vec::new();
        if !self.bytes.is_empty() {
            let line = std::str::from_utf8(&self.bytes)
                .map_err(|err| Error::Stream(err.to_string()))?
                .to_string();
            self.bytes.clear();
            if let Some(event) = self.decode_line(&line) {
                events.push(event);
            }
        }

        if let Some(event) = self.flush_event() {
            events.push(event);
        }

        Ok(events)
    }

    fn next_line(&mut self) -> Result<Option<String>> {
        let Some(pos) = self
            .bytes
            .iter()
            .position(|byte| *byte == b'\n' || *byte == b'\r')
        else {
            return Ok(None);
        };

        let line = self.bytes.split_to(pos);
        let newline = self.bytes.get_u8();
        if newline == b'\r' && self.bytes.first() == Some(&b'\n') {
            self.bytes.advance(1);
        }

        let line = std::str::from_utf8(&line)
            .map_err(|err| Error::Stream(err.to_string()))?
            .to_string();
        Ok(Some(line))
    }

    fn decode_line(&mut self, line: &str) -> Option<ServerSentEvent> {
        if line.is_empty() {
            return self.flush_event();
        }

        if line.starts_with(':') {
            return None;
        }

        let (field, value) = line.split_once(':').unwrap_or((line, ""));
        let value = value.strip_prefix(' ').unwrap_or(value);

        match field {
            "event" => self.event = Some(value.to_string()),
            "data" => self.data.push(value.to_string()),
            "id" if !value.contains('\0') => self.last_event_id = Some(value.to_string()),
            "retry" => self.retry = value.parse().ok(),
            _ => {}
        }

        None
    }

    fn flush_event(&mut self) -> Option<ServerSentEvent> {
        if self.event.is_none()
            && self.data.is_empty()
            && self.last_event_id.is_none()
            && self.retry.is_none()
        {
            return None;
        }

        let event = ServerSentEvent {
            event: self.event.take(),
            data: self.data.join("\n"),
            id: self.last_event_id.clone(),
            retry: self.retry.take(),
        };
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
        assert!(decoder
            .push(Bytes::from_static(b"data: a\n"))
            .unwrap()
            .is_empty());
        let events = decoder.push(Bytes::from_static(b"data: b\n\n")).unwrap();

        assert_eq!(events[0].data, "a\nb");
    }

    #[test]
    fn keeps_last_event_id_across_events() {
        let mut decoder = SseDecoder::new();
        let events = decoder
            .push(Bytes::from_static(b"id: one\ndata: a\n\ndata: b\n\n"))
            .unwrap();

        assert_eq!(events[0].id.as_deref(), Some("one"));
        assert_eq!(events[1].id.as_deref(), Some("one"));
    }
}
