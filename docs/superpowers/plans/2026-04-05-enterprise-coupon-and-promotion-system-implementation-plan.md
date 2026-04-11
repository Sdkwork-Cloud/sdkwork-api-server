# Enterprise Coupon And Promotion System Implementation Plan

> Status: historical implementation plan. It was superseded by `docs/架构/166-*` and the 2026-04-10 full legacy coupon exit.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将当前 `sdkwork-api-router` 中的单表 `CouponCampaign` 能力升级为企业级优惠券与促销内核，落地模板、活动、预算、码池、验券、预占、核销、退款回滚、审计、监控与兼容迁移的完整闭环。

**Architecture:** 以 `sdkwork-api-domain-marketing + sdkwork-api-app-marketing + MarketingStore` 为中心建设营销内核；旧 `CouponCampaign` 降级为兼容投影视图。报价阶段执行 `validate`，下单阶段执行 `reserve`，支付确认阶段执行 `confirm redemption`，取消/退款阶段执行 `rollback`，并通过幂等键、出站事件、定时清理和审计流水保证事务安全、失败恢复和生产可观测性。

**Tech Stack:** Rust, Axum, sqlx, SQLite, PostgreSQL, libsql, MySQL, React, TypeScript, pnpm, cargo test

---

## Baseline

**Spec:** `docs/superpowers/specs/2026-04-05-enterprise-coupon-and-promotion-system-design.md`

**Current hard gaps to close first:**

- historical legacy coupon domain crate only had `CouponCampaign { code, discount_label, remaining, active }`-style single-record modeling, which could not express template, batch, shared-code, unique-code, budget, stacking, or audit semantics.
- `crates/sdkwork-api-app-commerce/src/lib.rs` 在报价和下单过程中直接解析 `discount_label`，并通过重写 `remaining` 来“消费”优惠券，缺少预占、幂等、回滚和并发保护。
- `crates/sdkwork-api-interface-admin/src/lib.rs` 当时仅暴露 legacy admin coupon CRUD routes，无法覆盖商用优惠券后台所需的模板、活动、批次、码池、核销明细与预算管理。
- `crates/sdkwork-api-interface-portal/src/lib.rs` 和 Portal 页面仍将优惠券视为一次性报价附属字段，没有标准化的验券、锁券、支付确认、退款释放链路。
- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-coupons/src/index.tsx` 仍是单表 CRUD 界面，`apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/pages/index.tsx` 仍是简单兑换入口，不具备运营、审计、退款与风控能力。

## File Map

### 新增营销内核

- `crates/sdkwork-api-domain-marketing/Cargo.toml`
- `crates/sdkwork-api-domain-marketing/src/lib.rs`
- `crates/sdkwork-api-domain-marketing/tests/coupon_template_records.rs`
- `crates/sdkwork-api-app-marketing/Cargo.toml`
- `crates/sdkwork-api-app-marketing/src/lib.rs`
- `crates/sdkwork-api-app-marketing/src/rules.rs`
- `crates/sdkwork-api-app-marketing/src/compat.rs`
- `crates/sdkwork-api-app-marketing/src/service.rs`
- `crates/sdkwork-api-app-marketing/src/fulfillment.rs`
- `crates/sdkwork-api-app-marketing/tests/validate_reserve_redeem.rs`
- `crates/sdkwork-api-app-marketing/tests/rollback_and_idempotency.rs`

### 存储与事务层

- `crates/sdkwork-api-storage-core/src/lib.rs`
- `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- `crates/sdkwork-api-storage-postgres/src/lib.rs`
- `crates/sdkwork-api-storage-libsql/src/lib.rs`
- `crates/sdkwork-api-storage-mysql/src/lib.rs`
- `crates/sdkwork-api-storage-sqlite/tests/marketing_roundtrip.rs`
- `crates/sdkwork-api-storage-postgres/tests/marketing_roundtrip.rs`
- `crates/sdkwork-api-storage-libsql/tests/marketing_roundtrip.rs`
- `crates/sdkwork-api-storage-mysql/tests/marketing_roundtrip.rs`

### 交易、账务、接口与前端

- `crates/sdkwork-api-domain-commerce/src/lib.rs`
- `crates/sdkwork-api-app-commerce/src/lib.rs`
- `crates/sdkwork-api-app-billing/src/lib.rs`
- `crates/sdkwork-api-interface-admin/src/lib.rs`
- `crates/sdkwork-api-interface-portal/src/lib.rs`
- `crates/sdkwork-api-app-jobs/src/lib.rs`
- `crates/sdkwork-api-app-rate-limit/src/lib.rs`
- `crates/sdkwork-api-observability/src/lib.rs`
- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts`
- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/src/index.ts`
- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-coupons/src/index.tsx`
- legacy coupon create dialog component（removed）
- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-coupons/src/page/CouponsRegistrySection.tsx`
- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-coupons/src/page/CouponsDetailPanel.tsx`
- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-coupons/src/page/CouponsDetailDrawer.tsx`
- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-coupons/src/page/shared.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/src/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/repository/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/services/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/pages/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/repository/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- `docs/api-reference/admin-api.md`
- `docs/api-reference/portal-api.md`
- `docs/zh/api-reference/admin-api.md`
- `docs/zh/api-reference/portal-api.md`

### 兼容与迁移文档

- `docs/superpowers/specs/2026-04-05-enterprise-coupon-and-promotion-system-design.md`
- `docs/superpowers/plans/2026-04-05-enterprise-coupon-and-promotion-system-implementation-plan.md`
- `docs/superpowers/specs/2026-04-05-enterprise-coupon-migration-playbook.md`

### 关键服务方法命名约定

- `validate_coupon_stack`
- `reserve_coupon_redemption`
- `confirm_coupon_redemption`
- `release_coupon_reservation`
- `rollback_coupon_redemption`
- `claim_coupon_code`
- `list_subject_coupon_assets`
- legacy coupon projection helper

### 关键数据表命名约定

- `ai_marketing_coupon_template`
- `ai_marketing_campaign`
- `ai_marketing_campaign_budget`
- `ai_marketing_coupon_code_batch`
- `ai_marketing_coupon_code`
- `ai_marketing_coupon_reservation`
- `ai_marketing_coupon_redemption`
- `ai_marketing_coupon_rollback`
- `ai_marketing_idempotency_key`
- `ai_marketing_outbox_event`

### Task 1: Freeze the current coupon model as a historical compatibility boundary

**Files:**
- Modify: historical legacy coupon domain compatibility layer
- Modify: historical legacy coupon app compatibility layer
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-app-commerce/src/lib.rs`
- Create: historical coupon compatibility tests

- [ ] **Step 1: 为 `CouponCampaign` 增加兼容层注释和边界说明**

目标：明确旧模型不再是业务真相，只保留旧接口兼容读写。

- [ ] **Step 2: 将 `list_active_coupons`、`persist_coupon`、`delete_coupon` 的职责限制为兼容层**

目标：禁止继续往旧层新增预算、核销、回滚等新语义。

- [ ] **Step 3: 在 `sdkwork-api-app-commerce` 中加注释和 TODO 边界**

目标：标记 `discount_label` 解析和 `remaining` 扣减只是过渡实现。

- [ ] **Step 4: 补兼容层测试**

Run: historical coupon compatibility tests for the campaign shim

Expected: legacy admin coupon compatibility behavior remains stable and the historical shim tests pass.

- [ ] **Step 5: Commit**

```bash
git add <historical legacy coupon compatibility files> crates/sdkwork-api-storage-core/src/lib.rs crates/sdkwork-api-app-commerce/src/lib.rs
git commit -m "refactor: freeze coupon campaign as compatibility layer"
```

### Task 2: Introduce canonical marketing domain records

**Files:**
- Create: `crates/sdkwork-api-domain-marketing/Cargo.toml`
- Create: `crates/sdkwork-api-domain-marketing/src/lib.rs`
- Create: `crates/sdkwork-api-domain-marketing/tests/coupon_template_records.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: 创建 `sdkwork-api-domain-marketing` crate**

目标：建立与旧 legacy coupon 兼容层并行的正式营销领域层。

- [ ] **Step 2: 定义核心聚合**

必须包含：`CouponTemplate`、`MarketingCampaign`、`CampaignBudget`、`CouponCodeBatch`、`CouponCode`、`CouponReservation`、`CouponRedemption`、`CouponRollback`、`MarketingOutboxEvent`。

- [ ] **Step 3: 定义生命周期枚举**

必须包含：模板状态、活动状态、批次状态、券码状态、预占状态、核销状态、回滚状态。

- [ ] **Step 4: 把 benefit 与 restriction 建模为显式类型**

必须包含：百分比折扣、固定金额、赠送额度、指定商品范围、首单限制、最小金额、互斥组、叠加规则。

- [ ] **Step 5: 写领域模型测试**

Run: `cargo test -p sdkwork-api-domain-marketing -- --nocapture`

Expected: 新域模型可独立编译，并覆盖状态流转和字段合法性。

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml crates/sdkwork-api-domain-marketing
git commit -m "feat: add marketing domain model"
```

### Task 3: Add marketing storage contracts, tables, and transaction helpers

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-libsql/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-mysql/src/lib.rs`
- Create: `crates/sdkwork-api-storage-sqlite/tests/marketing_roundtrip.rs`
- Create: `crates/sdkwork-api-storage-postgres/tests/marketing_roundtrip.rs`
- Create: `crates/sdkwork-api-storage-libsql/tests/marketing_roundtrip.rs`
- Create: `crates/sdkwork-api-storage-mysql/tests/marketing_roundtrip.rs`

- [ ] **Step 1: 在 `storage-core` 中引入 `MarketingStore` trait**

接口至少覆盖模板、活动、预算、批次、券码、预占、核销、回滚、幂等键和 outbox 事件。

- [ ] **Step 2: 为所有后端实现新表结构**

必须创建计划中的 10 张营销核心表，避免只在 SQLite 或 PostgreSQL 可用。

- [ ] **Step 3: 加唯一约束与高价值索引**

必须包含：`normalized_code_hash` 唯一键、`idempotency_key` 唯一键、`reservation_status + expires_at_ms` 索引、`campaign_id + status` 索引。

- [ ] **Step 4: 提供事务助手**

目标：让 `reserve -> order link -> confirm redemption -> rollback` 可以在单事务或受控补偿链路里执行。

- [ ] **Step 5: 为多存储后端补 roundtrip 测试**

Run: `cargo test -p sdkwork-api-storage-sqlite -p sdkwork-api-storage-postgres -p sdkwork-api-storage-libsql -p sdkwork-api-storage-mysql marketing_roundtrip -- --nocapture`

Expected: 各后端都能完成插入、查询、状态迁移和唯一键校验。

- [ ] **Step 6: Commit**

```bash
git add crates/sdkwork-api-storage-core/src/lib.rs crates/sdkwork-api-storage-sqlite crates/sdkwork-api-storage-postgres crates/sdkwork-api-storage-libsql crates/sdkwork-api-storage-mysql
git commit -m "feat: add marketing storage contracts and schema"
```

### Task 4: Build the validation, reservation, redemption, and rollback kernel

**Files:**
- Create: `crates/sdkwork-api-app-marketing/Cargo.toml`
- Create: `crates/sdkwork-api-app-marketing/src/lib.rs`
- Create: `crates/sdkwork-api-app-marketing/src/rules.rs`
- Create: `crates/sdkwork-api-app-marketing/src/compat.rs`
- Create: `crates/sdkwork-api-app-marketing/src/service.rs`
- Create: `crates/sdkwork-api-app-marketing/src/fulfillment.rs`
- Create: `crates/sdkwork-api-app-marketing/tests/validate_reserve_redeem.rs`
- Create: `crates/sdkwork-api-app-marketing/tests/rollback_and_idempotency.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: 创建 `sdkwork-api-app-marketing` crate**

目标：让营销业务逻辑从 `sdkwork-api-app-commerce` 中独立出来。

- [ ] **Step 2: 实现 `validate_coupon_stack`**

校验项必须覆盖：模板状态、活动时间窗、预算、适用对象、商品范围、首单限制、互斥组、堆叠顺序、单主体限次。

- [ ] **Step 3: 实现 `reserve_coupon_redemption`**

行为要求：锁定预算、锁定券码、记录 TTL、写入 `idempotency_key`，返回显式 reservation 记录。

- [ ] **Step 4: 实现 `confirm_coupon_redemption`**

行为要求：仅允许从 `reserved` 转入 `redeemed`，并产生不可变核销记录。

- [ ] **Step 5: 实现 `release_coupon_reservation` 和 `rollback_coupon_redemption`**

行为要求：支持超时释放、取消释放、全额退款回滚、部分退款回滚。

- [ ] **Step 6: 实现 legacy coupon projection helper**

目标：将旧 `CouponCampaign` 映射成共享码模板 + 活动 + 默认批次的兼容投影。

- [ ] **Step 7: 编写预占/核销/回滚/幂等测试**

Run: `cargo test -p sdkwork-api-app-marketing -- --nocapture`

Expected: 重放请求不重复核销；过期预占能释放；回滚能恢复预算与券码可用性。

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml crates/sdkwork-api-app-marketing
git commit -m "feat: add marketing validation reservation redemption kernel"
```

### Task 5: Connect coupon flows to commerce, payment, and billing closure

**Files:**
- Modify: `crates/sdkwork-api-domain-commerce/src/lib.rs`
- Modify: `crates/sdkwork-api-app-commerce/src/lib.rs`
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`
- Create: `crates/sdkwork-api-app-commerce/tests/marketing_checkout_closure.rs`

- [ ] **Step 1: 在订单模型中补营销链路字段**

必须包含：`coupon_reservation_id`、`coupon_redemption_id`、`marketing_campaign_id`、`subsidy_amount_minor`、`pricing_adjustment_id`。

- [ ] **Step 2: 报价阶段接入 `validate_coupon_stack`**

目标：`preview_portal_commerce_quote` 不再直接靠 `discount_label` 推断折扣。

- [ ] **Step 3: 下单阶段接入 `reserve_coupon_redemption`**

目标：创建订单前锁定券码和预算，而不是直接改写 `remaining`。

- [ ] **Step 4: 支付确认阶段接入 `confirm_coupon_redemption`**

目标：支付事件重放时不会重复发放权益或重复记账。

- [ ] **Step 5: 取消/退款阶段接入 `rollback_coupon_redemption`**

目标：实现与订单状态、商业账户、benefit lot、账务证据一致的退款闭环。

- [ ] **Step 6: 补支付回放与退款回滚测试**

Run: `cargo test -p sdkwork-api-app-commerce marketing_checkout_closure -- --nocapture`

Expected: 同一支付回调重复执行仍只核销一次；退款后预算和权益恢复正确。

- [ ] **Step 7: Commit**

```bash
git add crates/sdkwork-api-domain-commerce/src/lib.rs crates/sdkwork-api-app-commerce/src/lib.rs crates/sdkwork-api-app-billing/src/lib.rs crates/sdkwork-api-app-commerce/tests/marketing_checkout_closure.rs
git commit -m "feat: connect marketing with commerce and billing closure"
```

### Task 6: Add enterprise admin APIs while preserving legacy admin coupon compatibility

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Create: `crates/sdkwork-api-interface-admin/tests/marketing_coupon_routes.rs`
- Modify: `docs/api-reference/admin-api.md`
- Modify: `docs/zh/api-reference/admin-api.md`

- [ ] **Step 1: 新增营销后台路由**

必须包含：`/admin/marketing/coupon-templates`、`/campaigns`、`/budgets`、`/code-batches`、`/codes`、`/reservations`、`/redemptions`、`/rollbacks`。

- [ ] **Step 2: 保留 legacy admin coupon 兼容接口**

目标：旧页面和旧调用不立即失效，但内部转换到新投影层。

- [ ] **Step 3: 提供后台操作动作**

必须包含：创建模板、发布活动、生成码池、作废券码、手工释放预占、重放履约、查看核销链路。

- [ ] **Step 4: 写接口测试和文档**

Run: `cargo test -p sdkwork-api-interface-admin marketing_coupon_routes -- --nocapture`

Expected: 新后台路由可用，旧 legacy admin coupon compatibility route remains stable during migration.

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-interface-admin/src/lib.rs crates/sdkwork-api-interface-admin/tests/marketing_coupon_routes.rs docs/api-reference/admin-api.md docs/zh/api-reference/admin-api.md
git commit -m "feat: add enterprise marketing admin APIs"
```

### Task 7: Add portal validation, reservation, redemption, and reward-history APIs

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Create: `crates/sdkwork-api-interface-portal/tests/marketing_coupon_routes.rs`
- Modify: `docs/api-reference/portal-api.md`
- Modify: `docs/zh/api-reference/portal-api.md`

- [ ] **Step 1: 新增门户营销接口**

必须包含：`/portal/marketing/coupon-validations`、`/coupon-reservations`、`/coupon-redemptions/confirm`、`/coupon-redemptions/rollback`、`/my-coupons`、`/reward-history`。

- [ ] **Step 2: 报价和订单接口返回显式营销诊断**

必须包含：资格状态、拒绝原因、预占到期时间、补贴金额、回滚状态。

- [ ] **Step 3: 加用户与项目作用域边界**

目标：个人领券、项目用券、邀请奖励、账户历史都能被正确归属。

- [ ] **Step 4: 写路由测试和 API 文档**

Run: `cargo test -p sdkwork-api-interface-portal marketing_coupon_routes -- --nocapture`

Expected: Portal 侧能独立完成验券、锁券、核销确认和历史查询。

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-interface-portal/src/lib.rs crates/sdkwork-api-interface-portal/tests/marketing_coupon_routes.rs docs/api-reference/portal-api.md docs/zh/api-reference/portal-api.md
git commit -m "feat: add portal marketing coupon APIs"
```

### Task 8: Upgrade the admin control plane from single-table coupons to a marketing workbench

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/src/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-coupons/src/index.tsx`
- Modify: legacy coupon create dialog component（removed）
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-coupons/src/page/CouponsRegistrySection.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-coupons/src/page/CouponsDetailPanel.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-coupons/src/page/CouponsDetailDrawer.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-coupons/src/page/shared.tsx`
- Create: `apps/sdkwork-router-admin/tests/admin-marketing-workbench.test.mjs`

- [ ] **Step 1: 扩展 Admin API client 和类型**

新增模板、活动、批次、券码、预占、核销、回滚、预算视图类型和请求方法。

- [ ] **Step 2: 把现有 Coupons 页面改造成营销工作台**

至少包含四个工作视图：模板、活动、码池、核销记录。

- [ ] **Step 3: 保留旧 `/coupons` 概览卡片**

目标：老运营入口继续可见，但引导进入新的模板和码池管理视图。

- [ ] **Step 4: 增加运营风险可视化**

必须显示：预算耗尽风险、即将过期的 reservation、异常核销速度、手工回滚待处理项。

- [ ] **Step 5: 写前端测试**

Run: `pnpm --dir apps/sdkwork-router-admin test -- admin-marketing-workbench.test.mjs`

Expected: 工作台能消费新类型和新接口，旧入口未断。

- [ ] **Step 6: Commit**

```bash
git add apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/src/index.ts apps/sdkwork-router-admin/packages/sdkwork-router-admin-coupons apps/sdkwork-router-admin/tests/admin-marketing-workbench.test.mjs
git commit -m "feat: upgrade admin coupon module into marketing workbench"
```

### Task 9: Upgrade portal redeem, billing, and order history flows

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/src/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/repository/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/services/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/repository/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- Create: `apps/sdkwork-router-portal/tests/portal-marketing-coupon-flow.test.mjs`

- [ ] **Step 1: 扩展 Portal API client 与类型**

新增验券结果、预占结果、核销记录、奖励历史、退款回滚状态。

- [ ] **Step 2: 把 Credits 页改成显式验券 + 预占 + 提交闭环**

目标：用户输入码后先看到可用性和权益，再发起正式提交。

- [ ] **Step 3: 在 Billing 页展示营销证据**

必须展示：订单使用的券码、补贴金额、核销状态、退款后的回滚结果。

- [ ] **Step 4: 保持个人和项目维度隔离**

目标：邀请奖励、账户历史、项目充值、折扣订单都能区分所属主体。

- [ ] **Step 5: 写前端测试**

Run: `pnpm --dir apps/sdkwork-router-portal test -- portal-marketing-coupon-flow.test.mjs`

Expected: Portal 能完成验券、预占、支付确认后的成功展示，以及失败或回滚后的恢复展示。

- [ ] **Step 6: Commit**

```bash
git add apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/src/index.ts apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing apps/sdkwork-router-portal/tests/portal-marketing-coupon-flow.test.mjs
git commit -m "feat: upgrade portal coupon and billing experience"
```

### Task 10: Add jobs, observability, anti-abuse, and failure recovery

**Files:**
- Modify: `crates/sdkwork-api-app-jobs/src/lib.rs`
- Modify: `crates/sdkwork-api-app-rate-limit/src/lib.rs`
- Modify: `crates/sdkwork-api-observability/src/lib.rs`
- Create: `crates/sdkwork-api-app-jobs/tests/marketing_recovery_jobs.rs`

- [ ] **Step 1: 增加 reservation 过期清理任务**

目标：自动释放超时未支付订单占用的券码和预算。

- [ ] **Step 2: 增加 outbox 重试和死信处理**

目标：履约、账务同步、通知失败时可恢复，不靠人工查库修复。

- [ ] **Step 3: 给营销接口加流量控制**

至少限制：验券、领券、锁券、核销确认、回滚操作，维度覆盖 IP、用户、项目、券码哈希。

- [ ] **Step 4: 增加监控指标与结构化追踪**

必须包含：验证量、拒绝原因、活跃预占、过期预占、核销量、回滚量、预算耗尽、风控命中、validate/redeem 延迟。

- [ ] **Step 5: 补恢复与监控测试**

Run: `cargo test -p sdkwork-api-app-jobs marketing_recovery_jobs -- --nocapture`

Expected: 超时任务能释放资源；重试任务不会重复履约；监控埋点可被触发。

- [ ] **Step 6: Commit**

```bash
git add crates/sdkwork-api-app-jobs/src/lib.rs crates/sdkwork-api-app-rate-limit/src/lib.rs crates/sdkwork-api-observability/src/lib.rs crates/sdkwork-api-app-jobs/tests/marketing_recovery_jobs.rs
git commit -m "feat: add marketing recovery jobs and observability"
```

### Task 11: Execute compatibility migration and phased cutover

**Files:**
- Create: `docs/superpowers/specs/2026-04-05-enterprise-coupon-migration-playbook.md`
- Modify: `crates/sdkwork-api-app-marketing/src/compat.rs`
- Create: `crates/sdkwork-api-app-marketing/tests/legacy_coupon_projection.rs`

- [ ] **Step 1: 写迁移说明**

必须明确 Phase 1 影子写入、Phase 2 双读校验、Phase 3 新流量切换、Phase 4 旧模型冻结。

- [ ] **Step 2: 实现旧表到新内核的投影器**

目标：把历史 `ai_coupon_campaigns` 数据投影成共享码模板和默认活动。

- [ ] **Step 3: 做影子对账**

目标：对比旧报价结果与新验证结果，确保切换前误差可解释。

- [ ] **Step 4: 补兼容投影测试**

Run: `cargo test -p sdkwork-api-app-marketing legacy_coupon_projection -- --nocapture`

Expected: 历史优惠券可被稳定迁移，新旧结果在可接受范围内对齐。

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/specs/2026-04-05-enterprise-coupon-migration-playbook.md crates/sdkwork-api-app-marketing/src/compat.rs crates/sdkwork-api-app-marketing/tests/legacy_coupon_projection.rs
git commit -m "docs: add coupon migration playbook"
```

## Recommended Execution Order

1. Task 1: 冻结旧 `CouponCampaign` 兼容边界。
2. Task 2: 新建 `sdkwork-api-domain-marketing` 领域模型。
3. Task 3: 先打通 `MarketingStore` 和多后端存储表。
4. Task 4: 落地 `validate/reserve/redeem/rollback` 核心服务。
5. Task 5: 把交易、支付、账务链路接到新内核。
6. Task 6 and Task 7: 补后台与门户 API。
7. Task 8 and Task 9: 升级 Admin 与 Portal 前端。
8. Task 10: 加失败恢复、监控和风控限流。
9. Task 11: 做兼容迁移、影子对账和流量切换。

## First Production Slice

首个必须优先落地的上线切片不是 UI，而是后端交易安全内核：

1. `sdkwork-api-domain-marketing`
2. `MarketingStore`
3. `validate_coupon_stack`
4. `reserve_coupon_redemption`
5. `confirm_coupon_redemption`
6. `rollback_coupon_redemption`
7. Commerce/Billing 集成测试

只有这七项完成后，优惠券系统才具备商用级最小闭环。

## Exit Criteria

以下条件全部满足后，才视为企业级优惠券系统落地完成：

- 一个模板可以关联多个活动、多个批次、多个共享码和多个唯一券码。
- 报价、下单、支付确认、取消、退款、部分退款都能正确驱动 validate/reserve/redeem/rollback。
- 支付回调重放、前端重复提交、任务重试都不会造成重复核销或重复发放权益。
- 管理后台可以查看模板、活动、预算、码池、预占、核销、回滚和异常风险。
- 门户可以查看可用优惠、验证结果、奖励历史、订单营销证据和退款回滚结果。
- 系统具备速率限制、异常审计、监控指标、恢复任务和兼容迁移方案。
