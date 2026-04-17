// @ts-check

import { defineConfig } from 'vitepress';

const englishNav = [
  { text: 'Getting Started', link: '/getting-started/production-deployment' },
  { text: 'Architecture', link: '/architecture/software-architecture' },
  { text: 'API Reference', link: '/api-reference/overview' },
  { text: 'Operations', link: '/operations/install-layout' },
  { text: 'Reference', link: '/reference/api-compatibility' },
  { text: '中文', link: '/zh/' },
];

const chineseNav = [
  { text: '开始使用', link: '/zh/getting-started/production-deployment' },
  { text: '架构', link: '/zh/architecture/software-architecture' },
  { text: 'API 参考', link: '/zh/api-reference/overview' },
  { text: '运维', link: '/zh/operations/install-layout' },
  { text: '参考', link: '/zh/reference/api-compatibility' },
  { text: 'English', link: '/' },
];

const englishSidebar = [
  {
    text: 'Getting Started',
    items: [
      { text: 'Production Deployment', link: '/getting-started/production-deployment' },
      { text: 'Quickstart', link: '/getting-started/quickstart' },
      { text: 'Installation', link: '/getting-started/installation' },
      { text: 'Source Development', link: '/getting-started/source-development' },
      { text: 'Script Lifecycle', link: '/getting-started/script-lifecycle' },
      { text: 'Build and Packaging', link: '/getting-started/build-and-packaging' },
      { text: 'Release Builds', link: '/getting-started/release-builds' },
      { text: 'Runtime Modes', link: '/getting-started/runtime-modes' },
      { text: 'Public Portal', link: '/getting-started/public-portal' },
    ],
  },
  {
    text: 'Architecture',
    items: [
      { text: 'Software Architecture', link: '/architecture/software-architecture' },
      { text: 'Functional Modules', link: '/architecture/functional-modules' },
      { text: 'Runtime Modes Deep Dive', link: '/architecture/runtime-modes' },
    ],
  },
  {
    text: 'API Reference',
    items: [
      { text: 'Overview', link: '/api-reference/overview' },
      { text: 'Gateway API', link: '/api-reference/gateway-api' },
      { text: 'Admin API', link: '/api-reference/admin-api' },
      { text: 'Portal API', link: '/api-reference/portal-api' },
    ],
  },
  {
    text: 'Operations',
    items: [
      { text: 'Install Layout', link: '/operations/install-layout' },
      { text: 'Service Management', link: '/operations/service-management' },
      { text: 'Configuration', link: '/operations/configuration' },
      { text: 'Health and Metrics', link: '/operations/health-and-metrics' },
    ],
  },
  {
    text: 'Reference',
    items: [
      { text: 'API Compatibility', link: '/reference/api-compatibility' },
      { text: 'Repository Layout', link: '/reference/repository-layout' },
      { text: 'Build and Tooling', link: '/reference/build-and-tooling' },
      { text: 'Full Compatibility Matrix', link: '/api/compatibility-matrix' },
    ],
  },
];

const chineseSidebar = [
  {
    text: '开始使用',
    items: [
      { text: '生产部署', link: '/zh/getting-started/production-deployment' },
      { text: '快速开始', link: '/zh/getting-started/quickstart' },
      { text: '安装准备', link: '/zh/getting-started/installation' },
      { text: '源码运行', link: '/zh/getting-started/source-development' },
      { text: '脚本生命周期', link: '/zh/getting-started/script-lifecycle' },
      { text: '编译与打包', link: '/zh/getting-started/build-and-packaging' },
      { text: '发布构建', link: '/zh/getting-started/release-builds' },
      { text: '运行模式', link: '/zh/getting-started/runtime-modes' },
      { text: '公共门户', link: '/zh/getting-started/public-portal' },
    ],
  },
  {
    text: '架构',
    items: [
      { text: '软件架构', link: '/zh/architecture/software-architecture' },
      { text: '功能模块', link: '/zh/architecture/functional-modules' },
      { text: '运行模式详解', link: '/zh/architecture/runtime-modes' },
    ],
  },
  {
    text: 'API 参考',
    items: [
      { text: '总览', link: '/zh/api-reference/overview' },
      { text: '网关 API', link: '/zh/api-reference/gateway-api' },
      { text: '管理端 API', link: '/zh/api-reference/admin-api' },
      { text: '门户 API', link: '/zh/api-reference/portal-api' },
    ],
  },
  {
    text: '运维',
    items: [
      { text: '安装布局', link: '/zh/operations/install-layout' },
      { text: '服务管理', link: '/zh/operations/service-management' },
      { text: '配置说明', link: '/zh/operations/configuration' },
      { text: '健康检查与 Metrics', link: '/zh/operations/health-and-metrics' },
    ],
  },
  {
    text: '参考',
    items: [
      { text: 'API 兼容矩阵', link: '/zh/reference/api-compatibility' },
      { text: '仓库结构', link: '/zh/reference/repository-layout' },
      { text: '构建与工具链', link: '/zh/reference/build-and-tooling' },
      { text: '完整兼容矩阵（英文）', link: '/api/compatibility-matrix' },
    ],
  },
];

export default defineConfig({
  title: 'SDKWork API Server',
  description:
    'OpenAI-compatible gateway, admin control plane, public portal, and extension runtime.',
  lang: 'en-US',
  cleanUrls: true,
  lastUpdated: true,
  srcExclude: [
    'superpowers/**',
    'step/**',
    'review/**',
    'release/**',
    'plans/**',
    'prompts/**',
    '架构/**',
  ],
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
          copyright: 'Copyright (c) 2026 SDKWork',
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
          copyright: 'Copyright (c) 2026 SDKWork',
        },
      },
    },
  },
});
