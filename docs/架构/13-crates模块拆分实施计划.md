# Crates 模块拆分实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 建立统一的 crate 模块边界和大文件治理机制，持续消除 `crates` 目录中的超大 `lib.rs` 与职责混杂问题。

**Architecture:** 先建立统一治理规范，再按 crate 类型分批拆分。优先处理基础设施和高复用核心模块，再处理接口层、应用层、领域层和测试层，确保结构先收敛，再继续迭代业务能力。

**Tech Stack:** Rust、TypeScript/React、Axum、SQLx、OpenAPI、workspace crates

---

### Task 1: 建立统一治理基线

**Files:**
- Create: `docs/架构/12-crates模块规划与大文件治理规范.md`
- Create: `docs/架构/13-crates模块拆分实施计划.md`

- [x] Step 1: 盘点 `crates` 目录中超过 `1000` 行的源码文件
- [x] Step 2: 归纳 crate 类型与职责失衡模式
- [x] Step 3: 固化 `lib.rs`、模块命名、文件行数、目录结构规范
- [x] Step 4: 形成后续拆分统一模板

### Task 2: 收敛基础存储抽象层

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Create: `crates/sdkwork-api-storage-core/src/types.rs`
- Create: `crates/sdkwork-api-storage-core/src/admin_facets.rs`
- Create: `crates/sdkwork-api-storage-core/src/admin_store.rs`
- Create: `crates/sdkwork-api-storage-core/src/kernel_support.rs`
- Create: `crates/sdkwork-api-storage-core/src/identity_kernel_store.rs`
- Create: `crates/sdkwork-api-storage-core/src/account_kernel_store.rs`
- Create: `crates/sdkwork-api-storage-core/src/marketing_store.rs`
- Create: `crates/sdkwork-api-storage-core/src/account_transaction.rs`

- [x] Step 1: 按类型、facet、store trait、kernel transaction 分离模块
- [x] Step 2: 将根 `lib.rs` 收敛为导出入口
- [x] Step 3: 校验每个文件行数进入安全区间
- [x] Step 4: 把该结构作为 storage crate 样板

### Task 3: 继续拆分 Postgres 存储实现

**Files:**
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Create: `crates/sdkwork-api-storage-postgres/src/migrations.rs`
- Create: `crates/sdkwork-api-storage-postgres/src/postgres_migration_schema.rs`
- Create: `crates/sdkwork-api-storage-postgres/src/postgres_migration_compat.rs`
- Create: `crates/sdkwork-api-storage-postgres/src/postgres_migration_seed.rs`

- [ ] Step 1: 把 `run_migrations` 从根模块迁出
- [ ] Step 2: 按 schema / compatibility / seed 拆迁移逻辑
- [ ] Step 3: 让 `lib.rs` 只保留连接入口与导出
- [ ] Step 4: 继续清理剩余 account kernel 超大块

### Task 4: 对齐 SQLite 存储结构

**Files:**
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Create: `crates/sdkwork-api-storage-sqlite/src/migrations.rs`
- Create: `crates/sdkwork-api-storage-sqlite/src/catalog_store.rs`
- Create: `crates/sdkwork-api-storage-sqlite/src/commerce_checkout_store.rs`
- Create: `crates/sdkwork-api-storage-sqlite/src/commerce_finance_store.rs`
- Create: `crates/sdkwork-api-storage-sqlite/src/commerce_store_mappers.rs`
- Create: `crates/sdkwork-api-storage-sqlite/src/marketing_store.rs`
- Create: `crates/sdkwork-api-storage-sqlite/src/account_store.rs`
- Create: `crates/sdkwork-api-storage-sqlite/src/jobs_store.rs`
- Create: `crates/sdkwork-api-storage-sqlite/src/runtime_store.rs`

- [ ] Step 1: 以 Postgres 已成型结构为镜像模板
- [ ] Step 2: 把 SQLite 中 commerce / account / marketing / jobs / runtime 逐域拆开
- [ ] Step 3: 拆出 mapper、decoder、迁移逻辑
- [ ] Step 4: 将根文件压到 `1000` 行以下

### Task 5: 拆分 HTTP 接口总入口

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Create: `crates/sdkwork-api-interface-http/src/routes.rs`
- Create: `crates/sdkwork-api-interface-http/src/auth.rs`
- Create: `crates/sdkwork-api-interface-http/src/chat.rs`
- Create: `crates/sdkwork-api-interface-http/src/files.rs`
- Create: `crates/sdkwork-api-interface-http/src/responses.rs`
- Create: `crates/sdkwork-api-interface-http/src/webhook.rs`
- Create: `crates/sdkwork-api-interface-http/src/openapi.rs`
- Create: `crates/sdkwork-api-interface-http/src/error.rs`

- [ ] Step 1: 先按入口域拆 handlers
- [ ] Step 2: 把认证、错误映射、OpenAPI、Webhook 独立
- [ ] Step 3: 根模块只保留 router 装配和共享状态
- [ ] Step 4: 为后续真实支付回调与安全逻辑预留稳定模块边界

### Task 6: 拆分核心应用编排 crate

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-app-runtime/src/lib.rs`
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`

- [ ] Step 1: 按业务编排流程拆分 `gateway / billing / identity / runtime / routing`
- [ ] Step 2: 分离 provider 接入、策略决策、状态推进、回调处理
- [ ] Step 3: 消除应用层对接口 DTO 和存储细节的混杂引用
- [ ] Step 4: 形成 app crate 统一目录组织

### Task 7: 收敛领域与配置 crate

**Files:**
- Modify: `crates/sdkwork-api-domain-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-config/src/lib.rs`

- [ ] Step 1: 将 record、enum、policy、value object 分层
- [ ] Step 2: 把配置解析、默认值、环境变量映射独立
- [ ] Step 3: 保持 domain crate 专注于模型表达

### Task 8: 收敛测试文件规模

**Files:**
- Modify: `crates/*/tests/*.rs`

- [ ] Step 1: 将超大集成测试按业务主链路拆分
- [ ] Step 2: 避免一个测试文件覆盖过多 endpoint
- [ ] Step 3: 为最终统一回归测试做准备

### Task 9: 统一复核

**Files:**
- Modify: `docs/架构/11-超大文件拆分执行清单.md`
- Modify: `docs/架构/12-crates模块规划与大文件治理规范.md`
- Modify: `docs/架构/13-crates模块拆分实施计划.md`

- [ ] Step 1: 重新盘点超限文件清单
- [ ] Step 2: 记录已完成与剩余批次
- [ ] Step 3: 在最后统一执行编译与回归测试

## 2026-04-06 本轮新增进展

- 已完成 `crates/sdkwork-api-domain-routing/src/lib.rs` 拆分
  - 新增 `decision.rs`、`policy.rs`、`profile.rs`、`routing_support.rs`
  - 根 `lib.rs` 已收敛为导出入口

- 已完成 `crates/sdkwork-api-config/src/lib.rs` 拆分
  - 新增 `env_keys.rs`、`types.rs`、`loader.rs`、`http_exposure.rs`、`standalone_config.rs`、`config_support.rs`
  - 根 `lib.rs` 已收敛为导出入口

- 已完成 `crates/sdkwork-api-app-routing/src/lib.rs` 拆分
  - 新增 `route_inputs.rs`、`route_management.rs`、`route_selection.rs`、`candidate_selection.rs`、`routing_support.rs`
  - 根 `lib.rs` 已收敛为导出入口

- 已完成 `crates/sdkwork-api-app-billing/src/lib.rs` 拆分
  - 新增 `billing_inputs.rs`、`billing_events.rs`、`account_balance.rs`、`account_mutations.rs`、`commerce_credits.rs`、`billing_kernels.rs`、`pricing_lifecycle.rs`、`billing_summary.rs`、`billing_support.rs`
  - 根 `lib.rs` 已收敛为装配与导出入口

- 已完成 `crates/sdkwork-api-app-runtime/src/lib.rs` 拆分
  - 新增 `standalone_listener.rs`、`runtime_core.rs`、`runtime_builders.rs`、`rollout_models.rs`、`rollout_execution.rs`、`runtime_reload.rs`、`tests.rs`
  - 根 `lib.rs` 已收敛为装配与导出入口

- 已完成 `crates/sdkwork-api-app-identity/src/lib.rs` 拆分
  - 新增 `jwt_support.rs`、`identity_types.rs`、`admin_users.rs`、`api_key_groups.rs`、`gateway_api_keys.rs`、`portal_users.rs`、`portal_api_keys.rs`、`identity_support.rs`、`tests.rs`
  - 根 `lib.rs` 已收敛为装配与导出入口

- 已完成 `crates/sdkwork-api-storage-postgres/src/lib.rs` 二次拆分收口
  - 新增 `migrations.rs`、`postgres_migration_identity_schema.rs`、`postgres_migration_marketing_schema.rs`、`postgres_migration_routing_schema.rs`、`postgres_migration_billing_schema.rs`
  - 新增 `postgres_migration_commerce_jobs_schema.rs`、`postgres_migration_catalog_gateway_schema.rs`、`postgres_migration_runtime_schema.rs`、`postgres_migration_compat.rs`、`postgres_migration_seed.rs`
  - 新增 `account_kernel_store.rs`、`account_kernel_transaction.rs`
  - 根 `lib.rs` 已收敛为共享导入、模块装配与导出入口，账户内核事务与迁移编排已按职责拆离

- 已完成 `crates/sdkwork-api-app-gateway/src/lib.rs` 拆分
  - 新增 `gateway_types.rs`、`gateway_extension_host.rs`、`model_catalog.rs`、`gateway_provider_resolution.rs`、`gateway_cache.rs`、`gateway_routing.rs`
  - 新增 `gateway_execution_context.rs`、`request_context.rs`、`gateway_runtime_execution.rs`
  - 新增 `relay_chat.rs`、`relay_conversations.rs`、`relay_threads.rs`、`relay_responses.rs`、`relay_compute.rs`、`relay_containers.rs`
  - 新增 `relay_files_uploads.rs`、`relay_fine_tuning.rs`、`relay_assistants_realtime_webhooks.rs`、`relay_evals_batches.rs`、`relay_vector_stores.rs`、`relay_music_video.rs`、`tests.rs`
  - 根 `lib.rs` 已收敛为共享导入、模块声明、有限包装转发与公开导出入口，网关执行内核、路由缓存、扩展主机与各 OpenAI 资源 relay 已按职责分离

- 已完成 `crates/sdkwork-api-provider-openai/src/lib.rs` 拆分
  - 新增 `adapter_core.rs`、`dialog_resources.rs`、`media_resources.rs`、`control_resources.rs`、`openai_transport.rs`、`trait_impl.rs`
  - 根 `lib.rs` 已收敛为共享导入、模块声明、运输辅助内部导出与适配器公共导出入口
  - OpenAI 官方适配器已按“适配器核心 / 对话资源 / 媒体文件资源 / 控制面资源 / 传输辅助 / trait 分发”拆分，便于后续继续扩展资源端点与兼容头策略

- 已完成 `crates/sdkwork-api-extension-host/src/lib.rs` 拆分收口
  - 新增 `host_types.rs`、`extension_discovery.rs`、`extension_trust.rs`、`connector_runtime.rs`、`native_dynamic_runtime.rs`、`host_impl.rs`、`provider_invocation.rs`、`errors.rs`、`tests.rs`
  - 根 `lib.rs` 已收敛为共享导入、模块装配、内部复用导入与公共导出入口
  - 已修正测试模块错切、错误枚举 derive 缺失等拆分残留问题

- 已完成 `crates/sdkwork-api-storage-sqlite/src/lib.rs` 交易与基础设施大文件拆分
  - 新增迁移编排与 schema 模块：
    `migrations.rs`、`sqlite_migration_identity_schema.rs`、`sqlite_migration_marketing_schema.rs`、`sqlite_migration_routing_schema.rs`、`sqlite_migration_billing_schema.rs`、`sqlite_migration_commerce_jobs_schema.rs`、`sqlite_migration_catalog_gateway_schema.rs`、`sqlite_migration_catalog_gateway_compat.rs`、`sqlite_migration_runtime_schema.rs`、`sqlite_migration_legacy_compat.rs`
  - 新增存储与支持模块：
    `sqlite_support.rs`、`catalog_support.rs`、`catalog_store.rs`、`routing_store.rs`、`usage_billing_store.rs`、`tenant_store.rs`、`coupon_store.rs`、`jobs_store.rs`、`commerce_membership_store.rs`、`identity_store.rs`、`runtime_store.rs`、`admin_store_impl.rs`、`identity_kernel_store.rs`、`account_support.rs`、`account_kernel_store.rs`、`account_kernel_transaction.rs`、`marketing_support.rs`、`marketing_store_impl.rs`、`marketing_kernel_transaction.rs`、`tests.rs`
  - 复用并纳入 `commerce_store.rs`、`commerce_checkout_store.rs`、`commerce_finance_store.rs`、`commerce_store_mappers.rs`
  - 根 `lib.rs` 已收敛为共享导入、模块装配、迁移/支持模块导出、`SqliteAdminStore` 入口定义
  - 当前 `storage-sqlite` 各源码文件已控制在 `1000` 行以内，交易、账务、营销、身份、运行时、迁移兼容逻辑按职责分区

- 当前剩余重点拆分目标
  - `crates/sdkwork-api-interface-http/src/lib.rs`
  - 该模块仍为当前 crates 范围内主要超限文件，下一步按 OpenAPI、HTTP 暴露与指标、鉴权与请求上下文、路由装配、OpenAI 兼容入口、各资源域 handler、商业计费与用量记录等职责继续拆分

## 2026-04-06 interface-http 拆分完成

- 已完成 `crates/sdkwork-api-interface-http/src/lib.rs` 主体瘦身，`lib.rs` 仅保留 imports 与 include 装配。
- 已完成 OpenAPI 组装与 path 定义拆分：`gateway_openapi.rs`、`gateway_openapi_paths_*.rs`。
- 已完成 HTTP 接口实现按职责拆分为 `gateway_*` 多文件，覆盖 models/chat/conversations/threads/responses/files/uploads/fine_tuning/assistants/webhooks/realtime/evals/vector_stores 等域。
- 已修复拆分过程中的边界缺口，补齐 `with_state` handler、流式 helper、multipart helper、stateless relay、commercial support 等尾段实现。
- 当前 `crates/sdkwork-api-interface-http/src` 下正式 `.rs` 文件已全部压到 `1000` 行以内，最大文件为 `gateway_containers.rs`，约 `980` 行。
- 当前拆分形态采用 `include!(...)` 保持同模块作用域，先解决大文件与职责混杂问题；后续可在此基础上逐步提升为真正 `mod` 边界。
