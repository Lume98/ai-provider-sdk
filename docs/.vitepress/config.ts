import { defineConfig } from 'vitepress';

export default defineConfig({
  title: 'AI Provider SDK',
  description: 'Rust provider abstractions for AI applications',
  base: '/ai-provider-sdk/',
  cleanUrls: true,
  appearance: 'force-dark',
  themeConfig: {
    logo: { text: 'AI' },
    siteTitle: 'AI Provider SDK',
    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'Reference', link: '/reference/packages' },
      { text: 'GitHub', link: 'https://github.com/Lume98/ai-provider-sdk' },
    ],
    sidebar: [
      {
        text: 'Guide',
        items: [
          { text: 'Getting Started', link: '/guide/getting-started' },
          { text: 'Architecture', link: '/guide/architecture' },
          { text: 'OpenAI Provider', link: '/guide/openai' },
        ],
      },
      {
        text: 'Reference',
        items: [{ text: 'Packages', link: '/reference/packages' }],
      },
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/Lume98/ai-provider-sdk' },
    ],
    search: {
      provider: 'local',
    },
    footer: {
      message: 'Released under the Apache-2.0 License.',
      copyright: 'Copyright © 2026',
    },
  },
});
