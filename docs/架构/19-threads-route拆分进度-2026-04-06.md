# `threads_route` 拆分进度

## 本次目标

- 将 `crates/sdkwork-api-interface-http/tests/threads_route.rs` 从超大单文件重构为薄入口和职责化目录模块。
- 统一和 `runs_route`、`containers_route` 的结构，保持测试组织方式一致。
- 通过共享 helper 和上下文工厂降低重复代码，提升后续可维护性。

## 拆分结果

- 顶层薄入口：
  - `crates/sdkwork-api-interface-http/tests/threads_route.rs`
- 目录模块：
  - `crates/sdkwork-api-interface-http/tests/threads_route/mod.rs`
  - `crates/sdkwork-api-interface-http/tests/threads_route/route_support.rs`
  - `crates/sdkwork-api-interface-http/tests/threads_route/basic_routes.rs`
  - `crates/sdkwork-api-interface-http/tests/threads_route/stateless_relay.rs`
  - `crates/sdkwork-api-interface-http/tests/threads_route/stateful_core.rs`
  - `crates/sdkwork-api-interface-http/tests/threads_route/usage_billing.rs`
  - `crates/sdkwork-api-interface-http/tests/threads_route/upstream_fixtures.rs`

## 模块职责

- `mod.rs`
  - 只保留 imports、`support` 引入、子模块装配和 shared use。
- `route_support.rs`
  - 收口 JSON 解析、内存数据库初始化、通用 not found 断言、测试上下文工厂、upstream header 捕获状态。
- `basic_routes.rs`
  - 只覆盖 threads/message 基础 happy path 和 not found 断言。
- `stateless_relay.rs`
  - 只覆盖无状态网关直连上游的 relay 行为。
- `stateful_core.rs`
  - 只覆盖有状态 relay 主流程，以及无 usage 的 not found 约束。
- `usage_billing.rs`
  - 只覆盖 thread create、message retrieve、message create 的 usage/billing/route key/provider 选择断言。
- `upstream_fixtures.rs`
  - 只保留上游 mock handler 和 thread/message JSON fixture。

## 本次优化

- 将重复的 OpenAI not found 断言统一进共享 helper。
- 将 admin/gateway 初始化收口到 `LocalThreadsTestContext`，减少 stateful 测试重复初始化。
- 将上游 thread/message JSON fixture 下沉到 `upstream_fixtures.rs`，避免主测试文件混杂大量模拟实现。

## 行数控制

- `basic_routes.rs`: 281
- `stateless_relay.rs`: 183
- `stateful_core.rs`: 480
- `usage_billing.rs`: 617
- `route_support.rs`: 68
- `upstream_fixtures.rs`: 133

全部控制在 1000 行以内。

## 后续优先文件

- `crates/sdkwork-api-interface-http/tests/conversations_route.rs`
- `crates/sdkwork-api-interface-admin/tests/account_billing_routes.rs`
- `crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs`
- `crates/sdkwork-api-interface-http/tests/files_route.rs`
- `crates/sdkwork-api-interface-http/tests/vector_store_files_route.rs`

## 说明

- 本轮只做结构拆分与职责收口，不执行测试。
- 统一回归测试放在后续整体收尾阶段执行。
