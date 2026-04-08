# `containers_route` 拆分进度

## 本次目标

- 将 `crates/sdkwork-api-interface-http/tests/containers_route.rs` 从单文件超大测试拆分为薄入口和目录模块。
- 保留 `serial(extension_env)` 约束，避免拆分后破坏原有串行测试语义。
- 统一与 `evals_route`、`videos_route`、`runs_route` 的测试目录形态。

## 拆分结果

- 顶层薄入口：
  - `crates/sdkwork-api-interface-http/tests/containers_route.rs`
- 目录模块：
  - `crates/sdkwork-api-interface-http/tests/containers_route/mod.rs`
  - `crates/sdkwork-api-interface-http/tests/containers_route/route_support.rs`
  - `crates/sdkwork-api-interface-http/tests/containers_route/basic_routes.rs`
  - `crates/sdkwork-api-interface-http/tests/containers_route/stateless_relay.rs`
  - `crates/sdkwork-api-interface-http/tests/containers_route/stateful_core.rs`
  - `crates/sdkwork-api-interface-http/tests/containers_route/usage_billing.rs`
  - `crates/sdkwork-api-interface-http/tests/containers_route/upstream_fixtures.rs`

## 模块职责

- `mod.rs`
  - 只保留 imports、`support` 引入、子模块装配和 shared use。
- `route_support.rs`
  - 收口 JSON 解析、字节读取、内存数据库初始化、测试上下文工厂、共享 not found 断言、upstream header 捕获。
- `basic_routes.rs`
  - 基础容器路由和文件路由的 happy path / not found 覆盖。
- `stateless_relay.rs`
  - 无状态网关直连上游 relay 场景。
- `stateful_core.rs`
  - 有状态网关完整 relay 主流程，以及 not found/no-usage 约束。
- `usage_billing.rs`
  - 创建容器、容器文件检索、容器文件创建的 usage / billing / route key / provider 选择断言。
- `upstream_fixtures.rs`
  - 模拟 upstream handler 和二进制内容响应 fixture。

## 本次优化

- 将 header 捕获逻辑封装进 `UpstreamCaptureState` 方法，避免多处直接操作锁。
- 将 `local_containers_test_context` 统一为上下文工厂，减少 stateful 用例初始化重复。
- 将二进制内容读取抽到 `read_bytes`，避免文件内容断言重复样板代码。

## 行数控制

- `basic_routes.rs`: 308
- `stateless_relay.rs`: 192
- `stateful_core.rs`: 442
- `usage_billing.rs`: 620
- `route_support.rs`: 75
- `upstream_fixtures.rs`: 148

全部控制在 1000 行以内。

## 后续优先文件

- `crates/sdkwork-api-interface-http/tests/threads_route.rs`
- `crates/sdkwork-api-interface-http/tests/conversations_route.rs`
- `crates/sdkwork-api-interface-http/tests/files_route.rs`
- `crates/sdkwork-api-interface-http/tests/vector_store_files_route.rs`
- `crates/sdkwork-api-interface-http/tests/uploads_route.rs`

## 说明

- 本轮只做结构重组与职责收口，不运行测试。
- 统一回归测试留到后续整体收尾阶段执行。
