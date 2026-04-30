import { defineConfig } from 'vitepress'

const repo = process.env.GITHUB_REPOSITORY?.split('/')[1] ?? 'vendor-ai-sdk'
const isCi = process.env.GITHUB_ACTIONS === 'true'

export default defineConfig({
  title: 'vendor-ai-sdk',
  description: 'Handwritten Rust SDK for the OpenAI API',
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
          { text: '支持进度对照', link: '/api/progress' },
          { text: '资源总览', link: '/api/resources' },
          { text: 'Responses', link: '/api/responses' },
          { text: 'Chat Completions', link: '/api/chat' },
          { text: 'Files & Uploads', link: '/api/files' },
          { text: 'Streaming', link: '/api/streaming' },
          { text: 'Webhooks', link: '/api/webhooks' },
          { text: 'CLI', link: '/api/cli' }
        ]
      }
    ],
    socialLinks: [{ icon: 'github', link: 'https://github.com' }]
  }
})
