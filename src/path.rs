//! URL 路径片段编码工具。
//!
//! 确保 API 资源 ID（可能包含 `/`、空格等特殊字符）安全拼接到 URL 路径中。
//!
//! ## 安全字符集
//!
//! 遵循 RFC 3986 unreserved 字符加上部分 sub-delims，
//! 与 OpenAI Python SDK 的路径编码行为保持一致：
//!
//! - unreserved: `A-Z a-z 0-9 - . _ ~`
//! - sub-delims: `! $ & ' ( ) * + , ; = : @`
//!
//! 其他字节统一编码为 `%XX` 大写十六进制形式。

/// 对路径片段中的非安全字节做百分号编码。
///
/// ## 示例
///
/// ```text
/// "fine/tuned model" → "fine%2Ftuned%20model"
/// "a:b@c,d"         → "a:b@c,d"             (全部安全，原样保留)
/// ```
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

/// 判断单个字节是否属于路径安全字符集。
///
/// 安全条件：ASCII 字母数字，或在 RFC 3986 sub-delims / unreserved 允许集合中。
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
