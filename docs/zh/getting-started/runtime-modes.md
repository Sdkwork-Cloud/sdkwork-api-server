# 运行模式

SDKWork API Server 支持独立服务模式和桌面嵌入模式。

## Server Mode

特点：

- 各服务以独立二进制运行
- gateway、admin、portal 都通过 HTTP 暴露
- PostgreSQL 更适合共享部署
- 更适合浏览器访问和多人使用

## Embedded Mode

特点：

- 运行时可由嵌入宿主承载
- Tauri 可以直接承载与浏览器相同的 React 控制台
- 默认信任边界是本机 loopback
- SQLite 更适合本地桌面场景

## 浏览器与 Tauri 同时可用

开发时：

- `pnpm --dir console tauri:dev` 依赖 Vite dev server
- 同一个前端地址仍然可以从浏览器访问
- `start-workspace --tauri` 可以一次启动服务和桌面壳

进一步说明见：

- [运行模式详解](/architecture/runtime-modes)
