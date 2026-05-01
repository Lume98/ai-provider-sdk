import { defineConfig } from 'vitepress'

const repo = process.env.GITHUB_REPOSITORY?.split('/')[1] ?? 'openai-rust'
const isCi = process.env.GITHUB_ACTIONS === 'true'

export default defineConfig({
  title: 'openai-rust',
  description: '当前仓库实现的 Rust OpenAI SDK 文档',
  lang: 'zh-CN',
  base: isCi ? `/${repo}/` : '/',
  themeConfig: {
    nav: [
      { text: '指南', link: '/guide/getting-started' },
      { text: 'API', link: '/api/resources' }
    ],
    sidebar: [
      {
        text: '指南',
        items: [
          { text: '快速开始', link: '/guide/getting-started' },
          { text: '配置', link: '/guide/configuration' },
          { text: '错误处理', link: '/guide/errors' }
        ]
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
          { text: 'Streaming', link: '/api/streaming' }
        ]
      }
    ],
    socialLinks: [{ icon: 'github', link: 'https://github.com' }]
  }
})
