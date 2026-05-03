import { defineConfig } from 'vitepress'

const repo = process.env.GITHUB_REPOSITORY?.split('/')[1] ?? 'vendor-ai-sdk'
const isCi = process.env.GITHUB_ACTIONS === 'true'

export default defineConfig({
  title: 'vendor-ai-sdk',
  description: 'Rust SDK for OpenAI-compatible APIs',
  lang: 'zh-CN',
  base: isCi ? `/${repo}/` : '/',
  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/logo.svg' }],
  ],
  themeConfig: {
    nav: [
      { text: '指南', link: '/guide/overview' },
      { text: 'API', link: '/api/resources' },
    ],
    sidebar: [
      {
        text: '指南',
        items: [
          { text: '安装与使用总览', link: '/guide/overview' },
          { text: '快速开始', link: '/guide/getting-started' },
          { text: '配置', link: '/guide/configuration' },
          { text: '错误处理', link: '/guide/errors' },
        ],
      },
      {
        text: 'API',
        items: [
          { text: '资源总览', link: '/api/resources' },
          { text: 'Responses', link: '/api/responses' },
          { text: 'Chat Completions', link: '/api/chat' },
          { text: 'Files', link: '/api/files' },
          { text: 'Models', link: '/api/models' },
          { text: 'Embeddings', link: '/api/embeddings' },
          { text: 'Moderations', link: '/api/moderations' },
          { text: 'Streaming', link: '/api/streaming' },
          { text: 'CLI', link: '/api/cli' },
          { text: 'Webhooks', link: '/api/webhooks' },
        ],
      },
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/Lume98/vendor-ai-sdk' },
    ],
    footer: {
      message: 'Released under the MIT License.',
    },
    search: {
      provider: 'local',
    },
  },
})
