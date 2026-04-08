# extension_dispatch 拆分进度

日期：2026-04-07

## 背景

`crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs` 原文件接近 1500 行，包含 builtin host 校验、connector relay、native dynamic 加载与 reload、热更新监督、签名校验、上游模拟器与打包辅助函数等多类职责，已经超出可维护的测试文件尺度。

## 本次拆分目标

- 顶层入口保留为薄入口
- 目录模块保留 imports 与公共上下文
- 将 builtin host、connector relay、native dynamic reload、support 工具拆开
- 保持测试语义不变，只优化结构边界

## 拆分结果

顶层入口：

- `crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs`

目录模块：

- `crates/sdkwork-api-app-gateway/tests/extension_dispatch/mod.rs`

子模块：

- `crates/sdkwork-api-app-gateway/tests/extension_dispatch/builtin_host.rs`
- `crates/sdkwork-api-app-gateway/tests/extension_dispatch/connector_relay.rs`
- `crates/sdkwork-api-app-gateway/tests/extension_dispatch/native_dynamic_reload.rs`
- `crates/sdkwork-api-app-gateway/tests/extension_dispatch/support.rs`

## 职责边界

### builtin_host

负责内建 provider 扩展宿主的基础可见性与解析能力：

- 当前 provider 扩展是否已注册
- 按 extension id 解析 provider

### connector_relay

负责 connector/discovered extension 的转发链路：

- persisted extension instance base_url override
- disabled instance 阻断 relay
- discovered connector relay
- configured host cache 复用
- reload 失败时保留旧 host
- 强制签名策略下阻断 unsigned connector

### native_dynamic_reload

负责 native dynamic provider 相关能力：

- native dynamic relay
- native runtime reload / targeted reload
- extension tree 变更后的 hot reload supervision
- drain timeout 安全失败

### support

负责共享测试支持：

- upstream mock request/health handler
- connector-compatible upstream server
- discovered manifest 与 native manifest 构造
- 临时目录、环境变量守卫、签名与 SHA256 helper
- lifecycle / health 等待器

## 当前收益

- `extension_dispatch.rs` 已降为 2 行薄入口
- connector relay 与 native dynamic reload 关注点分离
- 共享 mock server、签名与文件工具统一沉到 `support.rs`
- 后续新增 extension dispatch 测试时，不需要继续把所有场景塞进单文件

## 本轮拆分后剩余 >1000 行真实文件

- `crates/sdkwork-api-app-routing/tests/simulate_route.rs`
- `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`

说明：

- `docs/plans/...` 与 `docs/superpowers/specs/...` 仍存在大文档文件，当前优先级继续放在真实代码与测试模块结构治理。
