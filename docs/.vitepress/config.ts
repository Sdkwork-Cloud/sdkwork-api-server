import { defineConfig } from 'vitepress';

const englishNav = [
  { text: 'Getting Started', link: '/getting-started/installation' },
  { text: 'Operations', link: '/operations/configuration' },
  { text: 'Reference', link: '/reference/api-compatibility' },
  { text: '中文', link: '/zh/' },
];

const chineseNav = [
  { text: '开始使用', link: '/zh/getting-started/installation' },
  { text: '运维', link: '/zh/operations/configuration' },
  { text: '参考', link: '/zh/reference/api-compatibility' },
  { text: 'English', link: '/' },
];

const englishSidebar = [
  {
    text: 'Getting Started',
    items: [
      { text: 'Installation', link: '/getting-started/installation' },
      { text: 'Source Development', link: '/getting-started/source-development' },
      { text: 'Release Builds', link: '/getting-started/release-builds' },
      { text: 'Runtime Modes', link: '/getting-started/runtime-modes' },
      { text: 'Public Portal', link: '/getting-started/public-portal' },
    ],
  },
  {
    text: 'Operations',
    items: [
      { text: 'Configuration', link: '/operations/configuration' },
      { text: 'Health and Metrics', link: '/operations/health-and-metrics' },
    ],
  },
  {
    text: 'Reference',
    items: [
      { text: 'API Compatibility', link: '/reference/api-compatibility' },
      { text: 'Repository Layout', link: '/reference/repository-layout' },
      { text: 'Detailed Runtime Modes', link: '/architecture/runtime-modes' },
      { text: 'Full Compatibility Matrix', link: '/api/compatibility-matrix' },
    ],
  },
];

const chineseSidebar = [
  {
    text: '开始使用',
    items: [
      { text: '安装准备', link: '/zh/getting-started/installation' },
      { text: '源码运行', link: '/zh/getting-started/source-development' },
      { text: 'Release 构建', link: '/zh/getting-started/release-builds' },
      { text: '运行模式', link: '/zh/getting-started/runtime-modes' },
      { text: 'Public Portal', link: '/zh/getting-started/public-portal' },
    ],
  },
  {
    text: '运维',
    items: [
      { text: '配置说明', link: '/zh/operations/configuration' },
      { text: '健康检查与 Metrics', link: '/zh/operations/health-and-metrics' },
    ],
  },
  {
    text: '参考',
    items: [
      { text: 'API 兼容矩阵', link: '/zh/reference/api-compatibility' },
      { text: '仓库结构', link: '/zh/reference/repository-layout' },
      { text: '运行模式详解', link: '/architecture/runtime-modes' },
      { text: '完整兼容矩阵', link: '/api/compatibility-matrix' },
    ],
  },
];

export default defineConfig({
  title: 'SDKWork API Server',
  description:
    'OpenAI-compatible gateway, control plane, extension runtime, and public portal.',
  lang: 'en-US',
  cleanUrls: true,
  lastUpdated: true,
  head: [['meta', { name: 'theme-color', content: '#0f766e' }]],
  themeConfig: {
    search: { provider: 'local' },
    socialLinks: [
      {
        icon: 'github',
        link: 'https://github.com/Sdkwork-Cloud/sdkwork-api-server',
      },
    ],
  },
  locales: {
    root: {
      label: 'English',
      lang: 'en-US',
      themeConfig: {
        nav: englishNav,
        sidebar: englishSidebar,
        outline: {
          label: 'On this page',
        },
        docFooter: {
          prev: 'Previous page',
          next: 'Next page',
        },
        footer: {
          message: 'SDKWork API Server documentation',
          copyright: 'Copyright © 2026 SDKWork',
        },
      },
    },
    zh: {
      label: '简体中文',
      lang: 'zh-CN',
      link: '/zh/',
      themeConfig: {
        nav: chineseNav,
        sidebar: chineseSidebar,
        outline: {
          label: '本页内容',
        },
        docFooter: {
          prev: '上一页',
          next: '下一页',
        },
        footer: {
          message: 'SDKWork API Server 文档',
          copyright: 'Copyright © 2026 SDKWork',
        },
      },
    },
  },
});
