# Crates 模块规划与大文件治理规范

更新日期：2026-04-06

## 背景问题

当前 `spring-ai-plus-business/apps/sdkwork-api-router/crates` 中，多个 crate 把大量实现直接堆叠在 `lib.rs`。这类结构在早期迭代速度快，但随着交易、路由、计费、营销、运行时、网关等能力不断叠加，会持续放大以下问题：

- `lib.rs` 同时承担类型定义、业务编排、存储实现、路由注册、OpenAPI、错误映射等多种职责，职责边界失真。
- 修改任何一个子能力都需要进入同一个超大文件，冲突率高，阅读和审查成本高。
- 单文件上下文过大，局部改动容易破坏无关逻辑，回归风险持续上升。
- 模块命名缺少稳定规则，后续新增功能只能继续往原文件堆，形成结构性债务。
- 测试文件也开始出现同样问题，单文件覆盖多条业务链路，不利于定位回归点。

结论很明确：继续把实现堆进 `lib.rs`，交易体系、支付闭环、路由网关、运行时治理都无法长期稳定演进。

## 总体治理目标

- 所有 crate 建立稳定、可复用的模块拆分模板。
- `lib.rs` 回归为“模块装配 + 导出入口”，不再承担主要业务实现。
- 单文件优先控制在 `300-800` 行。
- 超过 `1000` 行的文件必须进入拆分清单并优先治理。
- 新增功能默认落到对应业务模块，不允许继续向超大 `lib.rs` 追加。

## 文件行数控制规则

### 通用阈值

- 目标值：`300-800` 行
- 预警值：`800` 行
- 强制拆分值：`1000` 行

### lib.rs 专项阈值

- 目标值：`80-200` 行
- 上限值：`400` 行
- 超过 `400` 行时，必须把业务逻辑迁出，只保留：
  - `use`
  - `mod`
  - `pub use`
  - crate 级共享状态/错误类型
  - 极少量入口装配函数

## 模块设计原则

### 1. 按业务职责拆，不按“代码形态”拆

正确拆法：

- `commerce_store.rs`
- `marketing_store.rs`
- `payment_method.rs`
- `webhook.rs`
- `routing_profile.rs`

错误拆法：

- `utils.rs`
- `misc.rs`
- `helpers.rs`
- `temp.rs`
- `common.rs`（除非真的是明确、稳定、跨模块共享的基础抽象）

### 2. 一个文件只回答一个问题

每个模块必须能清楚回答：

- 它负责什么能力
- 它暴露什么接口
- 它依赖哪些上游模块
- 它不负责什么

### 3. 先切“完整职责”，再切零散工具

优先抽离可以独立理解和复用的整块职责：

- 一个完整 store
- 一组 handler
- 一个交易编排流程
- 一组 transaction executor

不要优先抽碎小工具函数，否则只会让调用关系更乱。

### 4. 新旧边界必须对齐

如果 SQLite、Postgres、HTTP、Admin、Portal 同时实现同一业务域，拆分结构应尽量镜像对齐，避免同一能力在不同 crate 中出现完全不同的目录组织。

## Crate 类型模板

## 一、`storage-*` crate

推荐结构：

- `lib.rs`
- `migrations.rs`
- `catalog_store.rs`
- `commerce_store.rs`
- `billing_store.rs`
- `marketing_store.rs`
- `routing_store.rs`
- `jobs_store.rs`
- `runtime_store.rs`
- `identity_store.rs`
- `tenant_store.rs`
- `*_mapper.rs`
- `*_support.rs`
- `transaction/*.rs` 或 `*_transaction.rs`

约束：

- `lib.rs` 只保留数据库连接初始化、模块导出、极少量统一入口。
- 表结构迁移与 seed 逻辑单独拆分，不允许继续塞在根文件。
- row mapper、decoder、converter 必须从 store 主逻辑中拆开。

## 二、`interface-*` crate

推荐结构：

- `lib.rs`
- `state.rs`
- `error.rs`
- `routes.rs`
- `openapi.rs`
- `auth.rs`
- `commerce.rs`
- `marketing.rs`
- `billing.rs`
- `jobs.rs`
- `runtime.rs`
- `http.rs`
- `dto/*.rs` 或按域拆分 DTO 文件

约束：

- handler、DTO、路由注册、OpenAPI 不能混在一个文件。
- webhook、安全校验、幂等、签名等逻辑必须独立成模块。

## 三、`app-*` crate

推荐结构：

- `lib.rs`
- `order.rs`
- `payment_method.rs`
- `payment_event.rs`
- `refund.rs`
- `settlement.rs`
- `coupon_state.rs`
- `routing_decision.rs`
- `accounting.rs`
- `reconciliation.rs`

约束：

- 只保留应用层编排，不混入存储细节和接口 DTO。
- 按业务流程拆，不按 command/query 名称机械拆。

## 四、`domain-*` crate

推荐结构：

- `lib.rs`
- `records.rs`
- `enums.rs`
- `value_objects.rs`
- `requests.rs`
- `responses.rs`
- `policy.rs`

约束：

- 域对象、枚举、值对象、策略规则分层清晰。
- 避免一个 `lib.rs` 塞满所有 record、enum、helper。

## 五、测试文件

推荐结构：

- 按 endpoint / 场景 / 业务链路拆分
- 单文件尽量不超过 `800` 行
- 集成测试按“一个主场景一文件”管理

## 命名规范

### Rust

- 按业务域命名：`payment_method.rs`、`commerce_store.rs`、`routing_store.rs`
- 表达事务边界：`account_transaction.rs`、`marketing_kernel_store.rs`
- 表达支撑职责：`postgres_support.rs`、`commerce_mapper.rs`

禁止：

- `misc.rs`
- `utils.rs`
- `helpers.rs`
- `temp.rs`
- `new.rs`
- `final.rs`

### TypeScript / React

- 页面入口：`index.tsx`
- 业务区块：`<domain><Feature>Sections.tsx`
- 抽屉弹窗：`<domain><Feature>Drawer.tsx`、`<domain><Feature>Dialog.tsx`
- 纯逻辑：`formatters.ts`、`mappers.ts`、`constants.ts`
- 国际化：`i18nTranslations<Domain>.ts`

## 当前优先治理清单

基于当前 `crates` 目录扫描结果，优先级如下：

### P0

- `crates/sdkwork-api-interface-http/src/lib.rs` `22170` 行
- `crates/sdkwork-api-storage-sqlite/src/lib.rs` `12513` 行
- `crates/sdkwork-api-app-gateway/src/lib.rs` `9635` 行
- `crates/sdkwork-api-storage-postgres/src/lib.rs` `3979` 行

### P1

- `crates/sdkwork-api-extension-host/src/lib.rs` `3381` 行
- `crates/sdkwork-api-app-billing/src/lib.rs` `2610` 行
- `crates/sdkwork-api-app-identity/src/lib.rs` `2450` 行
- `crates/sdkwork-api-app-runtime/src/lib.rs` `2247` 行
- `crates/sdkwork-api-provider-openai/src/lib.rs` `2175` 行
- `crates/sdkwork-api-domain-billing/src/lib.rs` `1440` 行
- `crates/sdkwork-api-config/src/lib.rs` `1422` 行
- `crates/sdkwork-api-app-routing/src/lib.rs` `1416` 行
- `crates/sdkwork-api-domain-routing/src/lib.rs` `1330` 行

### 已完成的样板化拆分

- `sdkwork-api-interface-admin`
- `sdkwork-api-interface-portal`
- `sdkwork-api-storage-postgres` 第一阶段
- `sdkwork-api-storage-core`
- `sdkwork-router-admin-core/i18n`

这些已拆文件将作为后续 crate 拆分模板，避免每个模块重新摸索目录结构。

## 本轮确定的标准动作

- 所有新功能优先新增子模块，不再直接写入超大 `lib.rs`
- 对超过 `1000` 行的文件持续治理，直到退出超限清单
- 对已经拆分的 crate，继续清理二次膨胀模块
- 每轮拆分结束后重新盘点超限文件，决定下一批目标

## 结果要求

- 目录结构能让新成员不看实现也能判断职责边界
- 文件命名稳定且可预测
- 模块间依赖方向清晰，避免跨层相互引用
- 后续交易体系、支付、退款、对账、Webhook 安全等功能都能按域落位，不再把复杂度堆进单文件
