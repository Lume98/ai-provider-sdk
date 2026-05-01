# 错误处理

统一错误类型：`openai_rust::Error`

- `ApiStatus { message, status, request_id, body }`：HTTP 状态码非 2xx
- `Timeout`：请求超时
- `Connection(String)`：网络或连接层异常
- `Config(String)`：配置不合法（例如缺失 `api_key`）
- `Url(ParseError)`：`base_url` 非法
- `HeaderValue(InvalidHeaderValue)`：header 值非法
- `Json(serde_json::Error)`：JSON 编解码失败
- `Io(std::io::Error)`：文件 I/O 失败
- `Stream(String)`：SSE 解码或流式事件错误

建议：

- 业务日志至少记录 `status`、`request_id`、`message`。
- 针对 `Timeout/Connection` 做重试，针对 `Config/Json` 直接失败。
