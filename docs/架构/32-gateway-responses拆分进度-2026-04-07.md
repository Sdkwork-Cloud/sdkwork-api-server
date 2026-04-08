# gateway_responses 拆分进度

日期：2026-04-07

## 背景

`crates/sdkwork-api-interface-http/src/gateway_responses.rs` 超过 1000 行，并且同样通过 `include!("gateway_responses.rs")` 直接注入 `lib.rs` 作用域。

该文件同时承载了：

- stateless response 路由
- 本地 response / chat completion helper
- 带商业准入与 usage 记录的 stateful response 路由
- SSE stream 本地回退响应

随着 response 交易与计费逻辑继续扩展，继续维持单文件会迅速恶化可维护性。

## 本次拆分目标

- 保留 `include!` 装配模式，避免影响 `lib.rs`
- 按操作语义拆分 stateful handler，而不是把所有 handler 再次塞回同一个文件
- 将 local helper 与 stream helper 下沉到共享文件
- 保证每个子文件边界清晰、规模可控

## 拆分结果

顶层入口：

- `crates/sdkwork-api-interface-http/src/gateway_responses.rs`

子文件：

- `crates/sdkwork-api-interface-http/src/gateway_responses/local_helpers.rs`
- `crates/sdkwork-api-interface-http/src/gateway_responses/stateless_handlers.rs`
- `crates/sdkwork-api-interface-http/src/gateway_responses/stateful_create.rs`
- `crates/sdkwork-api-interface-http/src/gateway_responses/stateful_queries.rs`
- `crates/sdkwork-api-interface-http/src/gateway_responses/stateful_mutations.rs`

## 职责边界

### local_helpers

负责本地 helper 与 stream 回退：

- create / compact / input tokens 本地响应
- invalid model / not found 映射
- local chat completion helper
- local response SSE body

### stateless_handlers

负责无状态 response 路由：

- create response
- count input tokens
- retrieve response
- list input items
- delete / cancel / compact

### stateful_create

负责创建型 response 路由：

- commercial admission
- stream / non-stream relay
- 本地回退
- usage record 与 token usage 记录

### stateful_queries

负责查询型 stateful 路由：

- input tokens
- retrieve
- input items list

### stateful_mutations

负责变更型 stateful 路由：

- delete
- cancel
- compact

## 当前收益

- `gateway_responses.rs` 已降为 5 行薄入口
- stateful create / query / mutation 关注点拆开，后续扩展不再互相污染
- local helper 与 SSE 回退逻辑集中，减少跨 handler 复制
- 子文件规模控制在 139 至 225 行之间

## 后续建议

- 继续优先治理仍然超过 1000 行的测试文件
- `sdkwork-api-interface-http/tests` 适合采用“顶层薄入口 + 目录模块 + support”模式
- `simulate_route/provider_health.rs` 适合继续按 snapshot / recovery / TTL / runtime 场景再拆一次
