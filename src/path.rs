//! 路径片段编码工具。确保资源 ID 安全拼接到 URL 路径中。

pub(crate) fn encode_path_segment(value: &str) -> String {
    let mut encoded = String::new();

    for byte in value.bytes() {
        if is_path_segment_safe(byte) {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }

    encoded
}

fn is_path_segment_safe(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'-' | b'.'
                | b'_'
                | b'~'
                | b'!'
                | b'$'
                | b'&'
                | b'\''
                | b'('
                | b')'
                | b'*'
                | b'+'
                | b','
                | b';'
                | b'='
                | b':'
                | b'@'
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_path_segment_like_python_sdk() {
        assert_eq!(
            encode_path_segment("fine/tuned model"),
            "fine%2Ftuned%20model"
        );
        assert_eq!(encode_path_segment("a:b@c,d"), "a:b@c,d");
    }
}
