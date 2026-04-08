# gateway_containers 拆分进度

日期：2026-04-07

## 背景

`crates/sdkwork-api-interface-http/src/gateway_containers.rs` 原文件超过 1000 行，并且通过 `include!("gateway_containers.rs")` 直接注入到 `lib.rs` 作用域中。

该文件长期混合了三类职责：

- 本地容器 fallback / response helper
- 无状态容器与容器文件 handler
- 带状态的容器与容器文件 handler

如果直接改造成普通 `mod gateway_containers;`，会破坏当前 `lib.rs` 通过 `include!` 共享作用域的装配方式，风险不必要地扩大。

## 本次拆分目标

- 保留顶层薄入口与 `include!` 装配方式
- 按请求上下文与资源边界拆分，而不是按行数硬切
- 将 container 与 container file handler 分离，降低单文件职责密度
- 控制子文件规模，避免拆分后再次逼近 1000 行

## 拆分结果

顶层入口：

- `crates/sdkwork-api-interface-http/src/gateway_containers.rs`

子文件：

- `crates/sdkwork-api-interface-http/src/gateway_containers/local_fallback.rs`
- `crates/sdkwork-api-interface-http/src/gateway_containers/stateless_containers.rs`
- `crates/sdkwork-api-interface-http/src/gateway_containers/stateless_container_files.rs`
- `crates/sdkwork-api-interface-http/src/gateway_containers/stateful_containers.rs`
- `crates/sdkwork-api-interface-http/src/gateway_containers/stateful_container_files.rs`

## 职责边界

### local_fallback

负责本地容器能力的共享 helper：

- container not found / file not found 映射
- local retrieve / delete / list / content result
- 本地 JSON / 二进制响应封装

### stateless_containers

负责无状态容器主资源 handler：

- create container
- list containers
- retrieve container
- delete container

### stateless_container_files

负责无状态容器文件 handler：

- create file
- list files
- retrieve file
- delete file
- file content

### stateful_containers

负责带 `GatewayApiState` 的容器主资源 handler：

- relay from store
- local fallback
- usage record

### stateful_container_files

负责带 `GatewayApiState` 的容器文件 handler：

- relay from store
- local fallback
- usage record
- file content passthrough / local binary response

## 当前收益

- `gateway_containers.rs` 已降为 5 行薄聚合入口
- 保留了 `lib.rs` 现有 `include!` 架构，不引入额外装配风险
- container 与 container file 的 stateless/stateful 逻辑边界清晰
- 子文件规模收敛到 89 至 387 行，后续继续扩展时更易维护

## 后续建议

- 按同样策略继续治理 `sdkwork-api-interface-http` 下的 `gateway_responses.rs`、`gateway_vector_store_files.rs`
- 对 `include!` 型大文件优先采用“薄入口 + 子 include 文件”方案
- 对测试大文件优先采用“顶层薄入口 + 目录模块 + support 下沉”方案
