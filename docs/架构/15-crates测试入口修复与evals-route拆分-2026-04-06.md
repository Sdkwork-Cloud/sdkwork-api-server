# crates 测试入口修复与 `evals_route` 拆分进度

## 本轮完成

- 修复已拆分集成测试的顶层入口形态，补齐薄入口文件，避免仅保留 `tests/<name>/mod.rs` 时测试发现不稳定：
  - `crates/sdkwork-api-interface-http/tests/chat_route.rs`
  - `crates/sdkwork-api-interface-http/tests/fine_tuning_route.rs`
  - `crates/sdkwork-api-interface-http/tests/responses_route.rs`
  - `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
  - `crates/sdkwork-api-provider-openai/tests/http_execution.rs`
- 完成 `crates/sdkwork-api-interface-http/tests/evals_route.rs` 的目录模块化拆分，并将超大单文件替换为薄入口：
  - `crates/sdkwork-api-interface-http/tests/evals_route.rs`
  - `crates/sdkwork-api-interface-http/tests/evals_route/mod.rs`
  - `crates/sdkwork-api-interface-http/tests/evals_route/route_support.rs`
  - `crates/sdkwork-api-interface-http/tests/evals_route/basic_routes.rs`
  - `crates/sdkwork-api-interface-http/tests/evals_route/stateless_relay.rs`
  - `crates/sdkwork-api-interface-http/tests/evals_route/extended_routes.rs`
  - `crates/sdkwork-api-interface-http/tests/evals_route/stateful_core.rs`
  - `crates/sdkwork-api-interface-http/tests/evals_route/usage_billing.rs`
  - `crates/sdkwork-api-interface-http/tests/evals_route/upstream_fixtures.rs`

## 当前拆分边界

- `mod.rs`
  - 只保留 imports、共享 support 引入、子模块装配
- `route_support.rs`
  - 共享 JSON 读取、OpenAI not found 断言、内存库、上下文构造
- `basic_routes.rs`
  - 基础路由与默认 not found 断言
- `stateless_relay.rs`
  - 无状态核心中继路径
- `extended_routes.rs`
  - 扩展 run 路由与无状态错误封装
- `stateful_core.rs`
  - 有状态核心中继与无使用记录 not found 场景
- `usage_billing.rs`
  - 计费、usage、decision log 相关场景
- `upstream_fixtures.rs`
  - 上游 mock handler

## 文件规模核对

- `crates/sdkwork-api-interface-http/tests/evals_route/basic_routes.rs`: 360 行
- `crates/sdkwork-api-interface-http/tests/evals_route/extended_routes.rs`: 266 行
- `crates/sdkwork-api-interface-http/tests/evals_route/stateful_core.rs`: 444 行
- `crates/sdkwork-api-interface-http/tests/evals_route/usage_billing.rs`: 653 行

全部控制在 1000 行以内。

## 仍待处理的大文件

- `crates/sdkwork-api-interface-http/tests/videos_route.rs`: 2870 行
- `crates/sdkwork-api-app-routing/tests/simulate_route.rs`: 2718 行
- `crates/sdkwork-api-interface-portal/tests/portal_commerce.rs`: 2267 行
- `crates/sdkwork-api-interface-http/tests/runs_route.rs`: 1802 行
- `crates/sdkwork-api-interface-http/tests/containers_route.rs`: 1681 行
- `crates/sdkwork-api-interface-http/tests/threads_route.rs`: 1668 行
- `crates/sdkwork-api-interface-http/tests/conversations_route.rs`: 1665 行
- `crates/sdkwork-api-interface-admin/tests/account_billing_routes.rs`: 1579 行
- `crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs`: 1497 行
- `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision.rs`: 1378 行

## 下一步建议

- 优先继续拆 `crates/sdkwork-api-interface-http/tests/videos_route.rs`
  - 该文件体量最大，且与已经完成的 HTTP route 测试拆分模式最接近，收益最高
- 拆分时继续沿用“薄入口 + 目录模块”标准形态
  - 避免只保留子目录 `mod.rs`
  - 保持 Cargo 集成测试入口稳定
