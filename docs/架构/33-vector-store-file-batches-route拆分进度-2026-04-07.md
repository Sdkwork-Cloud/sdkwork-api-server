# vector_store_file_batches_route 拆分进度

日期：2026-04-07

## 背景

`crates/sdkwork-api-interface-http/tests/vector_store_file_batches_route.rs` 超过 1000 行，同时混合了基础路由断言、stateless relay、stateful relay、provider 选择、缺失 usage 场景和上游 mock handler。

继续维持单文件会让测试语义模糊，也不利于继续补充 vector store file batch 场景。

## 本次拆分目标

- 顶层测试入口降为薄入口
- 按测试语义拆分场景，而不是按行数硬切
- 将本地 test support 与上游 mock handler 下沉
- 继续复用 `tests/support/mod.rs` 的共享 helper

## 拆分结果

顶层入口：

- `crates/sdkwork-api-interface-http/tests/vector_store_file_batches_route.rs`

目录模块：

- `crates/sdkwork-api-interface-http/tests/vector_store_file_batches_route/mod.rs`

子模块：

- `crates/sdkwork-api-interface-http/tests/vector_store_file_batches_route/basic_routes.rs`
- `crates/sdkwork-api-interface-http/tests/vector_store_file_batches_route/stateless_relay.rs`
- `crates/sdkwork-api-interface-http/tests/vector_store_file_batches_route/stateful_relay.rs`
- `crates/sdkwork-api-interface-http/tests/vector_store_file_batches_route/provider_selection_retrieve.rs`
- `crates/sdkwork-api-interface-http/tests/vector_store_file_batches_route/provider_selection_create.rs`
- `crates/sdkwork-api-interface-http/tests/vector_store_file_batches_route/missing_usage.rs`
- `crates/sdkwork-api-interface-http/tests/vector_store_file_batches_route/route_support.rs`

## 职责边界

### basic_routes

负责无状态本地路由的基础结果断言：

- create / retrieve / cancel / files
- unknown batch not found

### stateless_relay

负责 stateless upstream relay 场景：

- OpenAI compatible provider relay
- authorization 透传
- retrieve / cancel / files relay

### stateful_relay

负责 stateful gateway relay 主路径：

- admin/provider/credential 装配
- gateway api key 访问
- provider upstream relay

### provider_selection_retrieve

负责 retrieve 场景的 route key / provider selection 断言。

### provider_selection_create

负责 create 场景的 route key / provider selection 断言。

### missing_usage

负责 not found 且不应落 usage 的保护性测试。

### route_support

负责本地辅助能力：

- `read_json`
- `memory_pool`
- `UpstreamCaptureState`
- upstream mock handlers

## 当前收益

- 顶层入口收敛为显式 `#[path = ...]` 薄入口，避免同名模块解析歧义
- 所有子文件都控制在 225 行以内
- vector store file batch 场景已经按职责解耦，后续继续补充不会重新堆回单文件

## 当前剩余超限文件

- `crates/sdkwork-api-provider-openai/tests/http_execution/evals_batches_vector_stores.rs`
- `crates/sdkwork-api-interface-portal/tests/marketing_coupon_routes.rs`
