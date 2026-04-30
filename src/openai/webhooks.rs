use reqwest::header::HeaderMap;
use ring::hmac;

use super::WebhooksResource;
use crate::Error;

impl WebhooksResource {
    pub fn verify_signature(
        &self,
        secret: &str,
        payload: &[u8],
        headers: &HeaderMap,
    ) -> Result<(), Error> {
        let _ = &self.core;
        let signature = headers
            .get("webhook-signature")
            .or_else(|| headers.get("OpenAI-Signature"))
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| Error::WebhookVerification("missing signature header".into()))?;
        let timestamp = headers
            .get("webhook-timestamp")
            .or_else(|| headers.get("OpenAI-Timestamp"))
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| Error::WebhookVerification("missing timestamp header".into()))?;
        let expected = webhook_signature(secret, timestamp, payload);
        let provided = signature
            .split(',')
            .find_map(|part| part.trim().strip_prefix("v1="))
            .unwrap_or(signature.trim());
        if constant_time_eq(expected.as_bytes(), provided.as_bytes()) {
            Ok(())
        } else {
            Err(Error::WebhookVerification("signature mismatch".into()))
        }
    }
}

fn webhook_signature(secret: &str, timestamp: &str, payload: &[u8]) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());
    let mut body = timestamp.as_bytes().to_vec();
    body.push(b'.');
    body.extend_from_slice(payload);
    hex_lower(hmac::sign(&key, &body).as_ref())
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0xf) as usize] as char);
    }
    out
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0_u8;
    for (a, b) in left.iter().zip(right.iter()) {
        diff |= a ^ b;
    }
    diff == 0
}
