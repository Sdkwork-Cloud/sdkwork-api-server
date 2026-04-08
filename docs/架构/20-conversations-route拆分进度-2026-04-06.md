# `conversations_route` 拆分进度

## 本次目标

- 将 `crates/sdkwork-api-interface-http/tests/conversations_route.rs` 从超大单文件重构为薄入口和职责化目录模块。
- 统一与 `threads_route`、`runs_route`、`containers_route` 的测试结构，保持命名和边界一致。
- 将 responses 能力下的 conversation/item 测试按行为维度拆开，避免继续膨胀。

## 拆分结果

- 顶层薄入口：
  - `crates/sdkwork-api-interface-http/tests/conversations_route.rs`
- 目录模块：
  - `crates/sdkwork-api-interface-http/tests/conversations_route/mod.rs`
  - `crates/sdkwork-api-interface-http/tests/conversations_route/route_support.rs`
  - `crates/sdkwork-api-interface-http/tests/conversations_route/basic_routes.rs`
  - `crates/sdkwork-api-interface-http/tests/conversations_route/stateless_relay.rs`
  - `crates/sdkwork-api-interface-http/tests/conversations_route/stateful_core.rs`
  - `crates/sdkwork-api-interface-http/tests/conversations_route/usage_billing.rs`
  - `crates/sdkwork-api-interface-http/tests/conversations_route/upstream_fixtures.rs`

## 模块职责

- `mod.rs`
  - 只保留 imports、`support` 引入、子模块装配和 shared use。
- `route_support.rs`
  - 收口 JSON 解析、内存数据库初始化、统一 not found 断言、上下文工厂、upstream header 捕获状态。
- `basic_routes.rs`
  - conversation / item 的基础 happy path 和 not found 断言。
- `stateless_relay.rs`
  - 无状态网关对 conversation 路由的 relay 覆盖。
- `stateful_core.rs`
  - 有状态 relay 主流程和无 usage 的 not found 约束。
- `usage_billing.rs`
  - conversation create、item retrieve、item create 的 usage/billing/route key/provider 选择断言。
- `upstream_fixtures.rs`
  - 上游 mock handler 与 conversation/item JSON fixture。

## 本次优化

- 将重复的 conversation/item not found 断言统一到共享 helper。
- 将 admin/gateway 初始化收口为 `LocalConversationsTestContext`，减少 stateful 测试重复代码。
- 将上游 conversation/item fixture 和 header 捕获逻辑下沉，避免主测试逻辑和 mock 实现耦合。

## 说明

- 本轮只做结构拆分与职责收口，不执行测试。
- 统一回归测试留待后续整体收尾阶段进行。
