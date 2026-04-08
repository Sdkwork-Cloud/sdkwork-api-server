# `videos_route` 拆分进度

## 本轮完成

- 完成 `crates/sdkwork-api-interface-http/tests/videos_route.rs` 从超大单文件到“薄入口 + 目录模块”的拆分
- 新增薄入口：
  - `crates/sdkwork-api-interface-http/tests/videos_route.rs`
- 新增目录模块：
  - `crates/sdkwork-api-interface-http/tests/videos_route/mod.rs`
  - `crates/sdkwork-api-interface-http/tests/videos_route/route_support.rs`
  - `crates/sdkwork-api-interface-http/tests/videos_route/basic_routes.rs`
  - `crates/sdkwork-api-interface-http/tests/videos_route/stateless_relay.rs`
  - `crates/sdkwork-api-interface-http/tests/videos_route/stateful_core.rs`
  - `crates/sdkwork-api-interface-http/tests/videos_route/native_dynamic.rs`
  - `crates/sdkwork-api-interface-http/tests/videos_route/canonical_routes.rs`
  - `crates/sdkwork-api-interface-http/tests/videos_route/usage_characters.rs`
  - `crates/sdkwork-api-interface-http/tests/videos_route/usage_video_lifecycle.rs`
  - `crates/sdkwork-api-interface-http/tests/videos_route/usage_canonical_outputs.rs`
  - `crates/sdkwork-api-interface-http/tests/videos_route/upstream_fixtures.rs`

## 模块职责

- `mod.rs`
  - imports、模块装配、共享 `use`
- `route_support.rs`
  - JSON/字节读取、not found 断言、内存库、测试上下文
- `basic_routes.rs`
  - 基础视频路由与 canonical 路由的基础返回测试
- `stateless_relay.rs`
  - 无状态核心视频路由 relay
- `stateful_core.rs`
  - 有状态核心视频路由 relay、not found without usage
- `native_dynamic.rs`
  - native dynamic 视频内容中继
- `canonical_routes.rs`
  - canonical characters/edits/extensions relay
- `usage_characters.rs`
  - 角色相关 usage / route key 选择
- `usage_video_lifecycle.rs`
  - create / remix / extend 生命周期计费
- `usage_canonical_outputs.rs`
  - edits / extensions canonical 输出计费
- `upstream_fixtures.rs`
  - 上游 mock handler

## 行数核对

- `basic_routes.rs`: 311 行
- `stateless_relay.rs`: 192 行
- `stateful_core.rs`: 331 行
- `native_dynamic.rs`: 149 行
- `canonical_routes.rs`: 256 行
- `usage_characters.rs`: 407 行
- `usage_video_lifecycle.rs`: 522 行
- `usage_canonical_outputs.rs`: 351 行
- `upstream_fixtures.rs`: 263 行

全部在 1000 行以内。

## 当前剩余超大文件

- `crates/sdkwork-api-app-routing/tests/simulate_route.rs`: 2718 行
- `crates/sdkwork-api-interface-portal/tests/portal_commerce.rs`: 2267 行
- `crates/sdkwork-api-interface-http/tests/runs_route.rs`: 1802 行
- `crates/sdkwork-api-interface-http/tests/containers_route.rs`: 1681 行
- `crates/sdkwork-api-interface-http/tests/threads_route.rs`: 1668 行
- `crates/sdkwork-api-interface-http/tests/conversations_route.rs`: 1665 行
- `crates/sdkwork-api-interface-admin/tests/account_billing_routes.rs`: 1579 行
- `crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs`: 1497 行
- `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision.rs`: 1378 行
- `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`: 1213 行
- `crates/sdkwork-api-interface-http/tests/files_route.rs`: 1102 行
- `crates/sdkwork-api-interface-http/tests/vector_store_files_route.rs`: 1096 行
- `crates/sdkwork-api-app-commerce/tests/marketing_checkout_closure.rs`: 1031 行
- `crates/sdkwork-api-interface-http/tests/uploads_route.rs`: 1015 行

## 下一步建议

- 优先继续拆 `crates/sdkwork-api-interface-http/tests/runs_route.rs`
  - 同属 HTTP route 测试，拆分模式可以直接复用
  - 与已完成的 `responses_route`、`fine_tuning_route`、`evals_route`、`videos_route` 风格保持一致
