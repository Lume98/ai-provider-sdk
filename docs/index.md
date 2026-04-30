---
layout: home

hero:
  name: vendor-ai-sdk
  text: Rust OpenAI SDK
  tagline: 基于当前仓库实现的功能文档
  actions:
    - theme: brand
      text: 快速开始
      link: /guide/getting-started
    - theme: alt
      text: 资源总览
      link: /api/resources

features:
  - title: OpenAI 风格资源树
    details: 从 client.responses 到 client.chat.completions，结构与调用路径清晰一致。
  - title: 流式能力
    details: 提供 TypedSseStream<T>，覆盖 Responses 和 Chat Completions 的 SSE 场景。
  - title: 文件与 Webhook
    details: 支持 files/uploads 及 webhook 签名校验，满足常见服务端集成。
---
