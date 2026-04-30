# CLI

仓库内置了一个轻量 `openai` CLI（`src/bin/openai.rs`）用于 smoke test。

## 示例

```bash
cargo run --bin openai -- models:list
cargo run --bin openai -- models:get gpt-4.1-mini
cargo run --bin openai -- files:list
cargo run --bin openai -- chat:create gpt-4.1-mini "hello"
cargo run --bin openai -- responses:create gpt-4.1-mini "hello"
```

## 额外测试命令

`raw:post` 当前内置支持：

- `/images/generations`
- `/batches`
