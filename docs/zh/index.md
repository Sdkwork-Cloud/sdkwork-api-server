---
layout: home

hero:
  name: SDKWork API Server
  text: OpenAI 兼容网关、控制平面、公共门户与扩展运行时
  tagline: 支持 Windows、Linux、macOS，支持 Rust 服务、浏览器控制台、Tauri 桌面壳，以及 SQLite / PostgreSQL。
  actions:
    - theme: brand
      text: 安装准备
      link: /zh/getting-started/installation
    - theme: alt
      text: 源码运行
      link: /zh/getting-started/source-development
    - theme: alt
      text: Release 构建
      link: /zh/getting-started/release-builds

features:
  - title: 跨平台启动
    details: 支持 Windows、Linux、macOS 的源码启动、浏览器模式、桌面模式以及分面启动。
  - title: 面向发布
    details: 支持构建 admin、gateway、portal 的 release 二进制，也支持浏览器静态资源和 Tauri 桌面包。
  - title: 面向外部用户
    details: 内置公共 portal，支持注册、登录、工作区查看与 API key 自助签发。
---

## 从这里开始

按你的目标选择入口：

- 首次接触项目：
  - 阅读 [安装准备](/zh/getting-started/installation)
- 从源码启动：
  - 阅读 [源码运行](/zh/getting-started/source-development)
- 构建发布产物：
  - 阅读 [Release 构建](/zh/getting-started/release-builds)
- 理解运行模式：
  - 阅读 [运行模式](/zh/getting-started/runtime-modes)
- 了解面向终端用户的入口：
  - 阅读 [Public Portal](/zh/getting-started/public-portal)

## 核心启动入口

完整栈浏览器模式：

```bash
node scripts/dev/start-workspace.mjs
```

完整栈桌面模式：

```bash
node scripts/dev/start-workspace.mjs --tauri
```

本地预览 docs：

```bash
pnpm --dir docs install
pnpm --dir docs dev
```
