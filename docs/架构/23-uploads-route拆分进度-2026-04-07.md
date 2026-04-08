# uploads_route 拆分进度

日期：2026-04-07

## 本次目标

将 `crates/sdkwork-api-interface-http/tests/uploads_route.rs` 从单文件测试实现拆分为目录化模块，统一到当前 HTTP route 测试的标准组织方式，降低后续上传交易相关场景扩展成本。

## 拆分结果

已完成以下结构调整：

- 顶层薄入口：`crates/sdkwork-api-interface-http/tests/uploads_route.rs`
- 目录模块入口：`crates/sdkwork-api-interface-http/tests/uploads_route/mod.rs`
- 共享辅助：`crates/sdkwork-api-interface-http/tests/uploads_route/route_support.rs`
- 基础路由：`crates/sdkwork-api-interface-http/tests/uploads_route/basic_routes.rs`
- 无状态转发：`crates/sdkwork-api-interface-http/tests/uploads_route/stateless_relay.rs`
- 有状态核心：`crates/sdkwork-api-interface-http/tests/uploads_route/stateful_core.rs`
- 计费与路由选择：`crates/sdkwork-api-interface-http/tests/uploads_route/usage_billing.rs`
- Upstream fixture：`crates/sdkwork-api-interface-http/tests/uploads_route/upstream_fixtures.rs`

## 模块职责

- `route_support.rs`
  - `read_json`
  - `assert_openai_not_found`
  - `memory_pool`
  - `LocalUploadTestContext`
  - `UpstreamCaptureState`
  - upload part multipart helper
- `basic_routes.rs`
  - uploads、parts、complete、cancel 基础 happy path
  - 上传会话不存在的本地错误断言
- `stateless_relay.rs`
  - stateless OpenAI 兼容上传转发
- `stateful_core.rs`
  - stateful OpenAI 兼容主路径
  - 缺失 usage 的 parts/complete/cancel 防护路径
- `usage_billing.rs`
  - created upload id 记账
  - upload route key 选路与 usage 归集
- `upstream_fixtures.rs`
  - uploads / parts / complete / cancel 上游 fixture

## 关键收益

- 上传主流程与 parts multipart 特殊处理已从单文件中解耦
- usage/billing 场景从基础路由中剥离，避免继续堆叠到一个测试文件
- 为后续继续拆分 `vector_store_file_batches_route`、`portal_commerce` 等大文件提供同类模板

## 当前状态

- 已完成代码拆分
- 暂未执行测试，保留到最后统一回归
