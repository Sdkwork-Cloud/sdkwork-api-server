# vector_store_files_route 拆分进度

日期：2026-04-07

## 本次目标

将 `crates/sdkwork-api-interface-http/tests/vector_store_files_route.rs` 从单体测试文件拆分为职责清晰的目录模块，统一到当前 HTTP route 集成测试的标准组织方式。

## 拆分结果

已完成以下结构调整：

- 顶层薄入口：`crates/sdkwork-api-interface-http/tests/vector_store_files_route.rs`
- 目录模块入口：`crates/sdkwork-api-interface-http/tests/vector_store_files_route/mod.rs`
- 共享辅助：`crates/sdkwork-api-interface-http/tests/vector_store_files_route/route_support.rs`
- 基础路由：`crates/sdkwork-api-interface-http/tests/vector_store_files_route/basic_routes.rs`
- 无状态转发：`crates/sdkwork-api-interface-http/tests/vector_store_files_route/stateless_relay.rs`
- 有状态核心：`crates/sdkwork-api-interface-http/tests/vector_store_files_route/stateful_core.rs`
- 计费与路由选择：`crates/sdkwork-api-interface-http/tests/vector_store_files_route/usage_billing.rs`
- Upstream fixture：`crates/sdkwork-api-interface-http/tests/vector_store_files_route/upstream_fixtures.rs`

## 模块职责

- `route_support.rs`
  - `read_json`
  - `assert_vector_store_file_not_found`
  - `memory_pool`
  - `LocalVectorStoreFilesTestContext`
  - `UpstreamCaptureState`
- `basic_routes.rs`
  - 向量库文件基础 happy path
  - 本地 not found 断言
- `stateless_relay.rs`
  - stateless OpenAI 兼容 provider 转发
- `stateful_core.rs`
  - stateful OpenAI 兼容主路径
  - 缺失 usage 的 retrieve/delete 保护路径
- `usage_billing.rs`
  - route key 选路
  - create usage 选路
  - created vector store file id 计费闭环
- `upstream_fixtures.rs`
  - create/list/retrieve/delete 以及 distinct-id fixture

## 关键收益

- 将 route key/provider 选择逻辑从基础路由中剥离
- 将 created-id 记账与普通 create/retrieve 路径分层，降低维护复杂度
- 对向量库文件路由的扩展点更清晰，后续新增退款、对账或支付关联测试时不会继续堆大单文件

## 当前状态

- 已完成代码拆分
- 暂未执行测试，保留到最后统一回归
