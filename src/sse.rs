use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures::Stream;
use serde::de::DeserializeOwned;

use crate::Error;

type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>;

pub struct SseJsonStream<T> {
    inner: ByteStream,
    buffer: Vec<u8>,
    done: bool,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Unpin for SseJsonStream<T> {}

impl<T> SseJsonStream<T> {
    pub fn new(inner: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static) -> Self {
        Self {
            inner: Box::pin(inner),
            buffer: Vec::new(),
            done: false,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> Stream for SseJsonStream<T>
where
    T: DeserializeOwned,
{
    type Item = Result<T, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }

        loop {
            if let Some(pos) = find_sse_boundary(&self.buffer) {
                let frame = String::from_utf8_lossy(&self.buffer[..pos]).to_string();
                let rest = self.buffer[pos..].to_vec();
                self.buffer = rest;
                match parse_sse_frame::<T>(&frame) {
                    Ok(SseFrame::Json(value)) => return Poll::Ready(Some(Ok(value))),
                    Ok(SseFrame::Done) => {
                        self.done = true;
                        self.buffer.clear();
                        return Poll::Ready(None);
                    }
                    Ok(SseFrame::Skip) => continue,
                    Err(err) => {
                        self.done = true;
                        return Poll::Ready(Some(Err(err)));
                    }
                }
            }

            match self.inner.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(bytes))) => {
                    self.buffer.extend_from_slice(&bytes);
                }
                Poll::Ready(Some(Err(err))) => {
                    self.done = true;
                    return Poll::Ready(Some(Err(Error::Http(err))));
                }
                Poll::Ready(None) => {
                    self.done = true;
                    if !self.buffer.is_empty() {
                        let frame = String::from_utf8_lossy(&self.buffer).to_string();
                        self.buffer.clear();
                        match parse_sse_frame::<T>(&frame) {
                            Ok(SseFrame::Json(value)) => return Poll::Ready(Some(Ok(value))),
                            Ok(SseFrame::Done) | Ok(SseFrame::Skip) => {}
                            Err(err) => return Poll::Ready(Some(Err(err))),
                        }
                    }
                    return Poll::Ready(None);
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

fn find_sse_boundary(buf: &[u8]) -> Option<usize> {
    let mut i = 0;
    while i + 1 < buf.len() {
        if buf[i] == b'\n' && buf[i + 1] == b'\n' {
            return Some(i + 2);
        }
        if i + 3 < buf.len()
            && buf[i] == b'\r'
            && buf[i + 1] == b'\n'
            && buf[i + 2] == b'\r'
            && buf[i + 3] == b'\n'
        {
            return Some(i + 4);
        }
        i += 1;
    }
    None
}

enum SseFrame<T> {
    Json(T),
    Done,
    Skip,
}

fn parse_sse_frame<T>(frame: &str) -> Result<SseFrame<T>, Error>
where
    T: DeserializeOwned,
{
    let mut data = String::new();
    for line in frame.lines() {
        let line = line.trim_end_matches('\r');
        let Some(payload) = line
            .strip_prefix("data: ")
            .or_else(|| line.strip_prefix("data:"))
        else {
            continue;
        };
        if payload == "[DONE]" {
            return Ok(SseFrame::Done);
        }
        if !payload.is_empty() {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(payload);
        }
    }

    if data.is_empty() {
        return Ok(SseFrame::Skip);
    }

    serde_json::from_str::<T>(&data)
        .map(SseFrame::Json)
        .map_err(|err| {
            Error::StreamProtocol(format!("invalid SSE JSON payload: {err}; payload={data}"))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{StreamExt, executor::block_on, stream};
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Event {
        text: String,
    }

    #[test]
    fn parses_split_sse_frames() {
        let stream = stream::iter(vec![
            Ok(Bytes::from_static(b"data: {\"text\":\"hel")),
            Ok(Bytes::from_static(b"lo\"}\r\n\r\ndata: [DONE]\r\n\r\n")),
        ]);
        let mut events = SseJsonStream::<Event>::new(stream);
        let out = block_on(async {
            let mut values = Vec::new();
            while let Some(item) = events.next().await {
                values.push(item.unwrap().text);
            }
            values
        });
        assert_eq!(out, vec!["hello"]);
    }
}
