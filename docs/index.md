---
layout: home

hero:
  name: openai-rust
  text: Rust OpenAI SDK
  tagline: 仅覆盖当前仓库已实现能力
  actions:
    - theme: brand
      text: 快速开始
      link: /guide/getting-started
    - theme: alt
      text: 资源总览
      link: /api/resources

features:
  - title: 小而明确的资源边界
    details: 当前仅实现 responses、chat completions、files、models、embeddings、moderations。
  - title: 统一请求入口
    details: 所有资源都支持默认调用和 `*_with_options` 扩展调用，透传 header/query/body/timeout。
  - title: 原生 SSE 流
    details: responses 与 chat completions 流式接口统一返回 `SseStream`，按 SSE 协议解码事件。
---
