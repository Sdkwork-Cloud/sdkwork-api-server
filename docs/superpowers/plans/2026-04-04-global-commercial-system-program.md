# Global Commercial System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** close the remaining commercial gaps in `sdkwork-api-router` so the platform has production-grade global payments, finance closure, durable multimodal execution, complete admin and portal commercial control planes, and explicit launch-readiness guarantees.

**Architecture:** preserve the existing plugin-first backend and React package-split products, but add a dedicated payment and finance kernel beside the current usage-settlement kernel. Implement money movement, recharge allocation, reconciliation, and async multimodal execution as first-class bounded subsystems instead of continuing to grow the current seeded commerce layer.

**Tech Stack:** Rust, Axum, sqlx, SQLite, PostgreSQL, Redis, React, TypeScript, pnpm, cargo test

---

### Task 1: Freeze the clean-slate commercial boundary and demote demo commerce behavior

**Files:**
- Modify: `docs/superpowers/specs/2026-04-03-advanced-commercial-system-readiness-review.md`
- Modify: `docs/superpowers/specs/2026-04-03-commercial-system-gap-assessment-and-target-solution.md`
- Modify: `docs/superpowers/specs/2026-04-03-commercial-pricing-architecture-design.md`
- Modify: `crates/sdkwork-api-app-commerce/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-commerce/src/lib.rs`
- Create: `docs/superpowers/specs/2026-04-04-global-commercial-system-design.md`

- [ ] Document that `sdkwork-api-app-commerce` remains the offer and quote layer, not the payment truth layer.
- [ ] Remove or mark seeded recharge and payment simulation behavior that would mislead future payment work.
- [ ] Add tests that preserve only safe offer-quote behavior inside `sdkwork-api-app-commerce`.
- [ ] Run: `cargo test -p sdkwork-api-app-commerce -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "docs: freeze commercial target-state boundaries"`

### Task 2: Introduce canonical payment and finance domain crates

**Files:**
- Create: `crates/sdkwork-api-domain-payments/Cargo.toml`
- Create: `crates/sdkwork-api-domain-payments/src/lib.rs`
- Create: `crates/sdkwork-api-domain-payments/tests/payment_records.rs`
- Create: `crates/sdkwork-api-domain-finance/Cargo.toml`
- Create: `crates/sdkwork-api-domain-finance/src/lib.rs`
- Create: `crates/sdkwork-api-domain-finance/tests/journal_records.rs`
- Modify: `Cargo.toml`

- [ ] Define canonical records for payment order, payment attempt, payment transaction, payment callback, refund order, dispute case, reconciliation batch, and invoice.
- [ ] Define immutable finance journal entry and journal line records plus drift and reconciliation summary shapes.
- [ ] Encode lifecycle enums for payment status, attempt status, callback verification status, refund status, dispute status, and reconciliation status.
- [ ] Run: `cargo test -p sdkwork-api-domain-payments -p sdkwork-api-domain-finance -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "feat: add payment and finance domain kernels"`

### Task 3: Add storage facets and transactional seams for payments and finance

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Create: `crates/sdkwork-api-storage-sqlite/tests/payment_finance_roundtrip.rs`
- Create: `crates/sdkwork-api-storage-postgres/tests/payment_finance_roundtrip.rs`
- Modify: `crates/sdkwork-api-storage-mysql/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-libsql/src/lib.rs`

- [ ] Add `PaymentStore` and `FinanceStore` facets rather than stuffing new operations into one unstructured trait surface.
- [ ] Add schema and CRUD for payment orders, attempts, transactions, callbacks, refunds, disputes, invoices, reconciliation batches, and finance journal entries.
- [ ] Add transaction-backed command seams for idempotent payment finalization and recharge grant linkage.
- [ ] Keep MySQL and LibSQL honest by returning explicit unsupported errors until real parity exists.
- [ ] Run: `cargo test -p sdkwork-api-storage-sqlite -p sdkwork-api-storage-postgres payment_finance_roundtrip -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "feat: add payment and finance storage seams"`

### Task 4: Implement the payment application kernel and provider plugin contract

**Files:**
- Create: `crates/sdkwork-api-app-payments/Cargo.toml`
- Create: `crates/sdkwork-api-app-payments/src/lib.rs`
- Create: `crates/sdkwork-api-app-payments/tests/payment_kernel.rs`
- Create: `crates/sdkwork-api-app-platform/src/lib.rs`
- Modify: `crates/sdkwork-api-app-runtime/src/lib.rs`
- Modify: `crates/sdkwork-api-config/src/lib.rs`

- [ ] Add the canonical payment orchestration kernel for order creation, attempt creation, callback intake, callback replay, refund submission, and reconciliation dispatch.
- [ ] Define a `PaymentGatewayPlugin` contract and registry with capability metadata, supported regions, and supported currencies.
- [ ] Add provider selection rules so gateway choice can depend on currency, country, product mode, and operator override.
- [ ] Wire Redis-backed idempotency and callback replay suppression into the payment kernel before any production gateway is enabled.
- [ ] Run: `cargo test -p sdkwork-api-app-payments -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "feat: add payment orchestration kernel and gateway registry"`

### Task 5: Land Stripe as the first production payment gateway

**Files:**
- Create: `crates/sdkwork-api-plugin-stripe/Cargo.toml`
- Create: `crates/sdkwork-api-plugin-stripe/src/lib.rs`
- Create: `crates/sdkwork-api-plugin-stripe/tests/stripe_checkout.rs`
- Modify: `crates/sdkwork-api-app-payments/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`

- [ ] Implement one-time recharge through Stripe Payment Intents or hosted checkout initiation.
- [ ] Implement overseas subscription and invoice initiation through the Stripe-facing payment kernel path.
- [ ] Verify webhook signature handling, idempotent callback processing, refund lifecycle, and dispute intake.
- [ ] Add sandbox configuration and smoke tests so the gateway can be verified without manual code edits.
- [ ] Run: `cargo test -p sdkwork-api-plugin-stripe -p sdkwork-api-app-payments stripe -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "feat: add stripe commercial payment gateway"`

### Task 6: Land Alipay and WeChat Pay as domestic gateways

**Files:**
- Create: `crates/sdkwork-api-plugin-alipay/Cargo.toml`
- Create: `crates/sdkwork-api-plugin-alipay/src/lib.rs`
- Create: `crates/sdkwork-api-plugin-alipay/tests/alipay_checkout.rs`
- Create: `crates/sdkwork-api-plugin-wechatpay/Cargo.toml`
- Create: `crates/sdkwork-api-plugin-wechatpay/src/lib.rs`
- Create: `crates/sdkwork-api-plugin-wechatpay/tests/wechatpay_checkout.rs`
- Modify: `crates/sdkwork-api-app-payments/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`

- [ ] Implement Alipay desktop web, mobile web, and QR-native initiation through the shared gateway contract.
- [ ] Implement WeChat Pay Native, H5, and JSAPI initiation through the shared gateway contract.
- [ ] Add refund, callback verification or decryption, and bill download or reconciliation hooks for both domestic gateways.
- [ ] Add sandbox or merchant-test configuration paths and smoke tests for both domestic gateways.
- [ ] Run: `cargo test -p sdkwork-api-plugin-alipay -p sdkwork-api-plugin-wechatpay -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "feat: add domestic commercial payment gateways"`

### Task 7: Close the money loop from confirmed payment to canonical account balance

**Files:**
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`
- Create: `crates/sdkwork-api-app-finance/Cargo.toml`
- Create: `crates/sdkwork-api-app-finance/src/lib.rs`
- Create: `crates/sdkwork-api-app-finance/tests/recharge_closure.rs`
- Modify: `crates/sdkwork-api-domain-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`

- [ ] Implement the bridge from confirmed payment order to finance journal to benefit-lot or balance grant allocation.
- [ ] Make recharge grants idempotent by payment business key so callback replay cannot double-credit accounts.
- [ ] Add drift detectors for payment-success versus balance-grant and payment-success versus finance-journal consistency.
- [ ] Run: `cargo test -p sdkwork-api-app-billing -p sdkwork-api-app-finance recharge_closure -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "feat: close payment-to-balance commercial loop"`

### Task 8: Finish the durable async multimodal execution kernel

**Files:**
- Modify: `crates/sdkwork-api-domain-jobs/src/lib.rs`
- Modify: `crates/sdkwork-api-app-jobs/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Create: `crates/sdkwork-api-interface-http/tests/async_multimodal_execution.rs`
- Create: `crates/sdkwork-api-app-jobs/tests/job_retries.rs`

- [ ] Extend the async job kernel from read and insert behavior into enqueue, claim, heartbeat, timeout, retry, callback reconciliation, asset finalize, and billing finalize behavior.
- [ ] Cut long-running image, video, audio, and music routes over to the async kernel with sync fast-path only where the provider truly supports it.
- [ ] Ensure actual usage evidence can finalize canonical request settlement after async completion instead of charging purely on estimate.
- [ ] Run: `cargo test -p sdkwork-api-app-jobs -p sdkwork-api-interface-http async_multimodal_execution -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "feat: complete async multimodal execution kernel"`

### Task 9: Expose canonical payment, finance, and job APIs in admin and portal

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Modify: `docs/api-reference/admin-api.md`
- Modify: `docs/api-reference/portal-api.md`
- Create: `crates/sdkwork-api-interface-admin/tests/payment_finance_routes.rs`
- Create: `crates/sdkwork-api-interface-portal/tests/payment_finance_routes.rs`

- [ ] Add admin APIs for payment orders, attempts, callbacks, refunds, disputes, invoices, reconciliation batches, and finance journal inspection.
- [ ] Add portal APIs for checkout initiation, payment method selection, order history, invoice history, recharge status, refund status, and async job evidence.
- [ ] Keep portal surfaces workspace-scoped and make admin investigation routes tenant-aware and operator-safe.
- [ ] Run: `cargo test -p sdkwork-api-interface-admin -p sdkwork-api-interface-portal payment_finance_routes -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "feat: add canonical payment and finance APIs"`

### Task 10: Sync the admin React packages to the canonical commercial backend

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-commercial/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-pricing/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-operations/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-overview/src/view-model.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/routeManifest.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/routes.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-payments/src/index.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-finance/src/index.tsx`
- Create: `apps/sdkwork-router-admin/tests/admin-payments-finance.test.mjs`

- [ ] Add typed admin SDK methods for payment, finance, invoice, refund, dispute, and reconciliation objects.
- [ ] Build dedicated payment and finance workbench packages instead of overloading one generic commercial page.
- [ ] Surface provider economics, callback replay failures, reconciliation drift, and async backlog in overview and operations modules.
- [ ] Run: `pnpm --dir apps/sdkwork-router-admin test`
- [ ] Checkpoint commit: `git commit -m "feat: upgrade admin commercial control plane"`

### Task 11: Sync the portal React packages to the canonical commercial backend

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/src/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commerce/src/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-account/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-settlements/src/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/routeManifest.ts`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-orders/src/index.tsx`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-invoices/src/index.tsx`
- Create: `apps/sdkwork-router-portal/tests/portal-payments-finance.test.mjs`

- [ ] Add typed portal SDK methods for checkout session creation, order history, invoice history, recharge completion, refund status, and async job asset inspection.
- [ ] Replace seeded or simulated payment state with canonical backend state in recharge and commerce flows.
- [ ] Surface regional payment options, pricing posture, settlements, invoices, and generated media jobs directly in the portal.
- [ ] Run: `pnpm --dir apps/sdkwork-router-portal test`
- [ ] Checkpoint commit: `git commit -m "feat: upgrade portal commercial self-service"`

### Task 12: Add plugin governance and capability visibility

**Files:**
- Modify: `crates/sdkwork-api-app-platform/src/lib.rs`
- Modify: `crates/sdkwork-api-app-runtime/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-operations/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/routeManifest.ts`
- Create: `crates/sdkwork-api-interface-admin/tests/plugin_governance_routes.rs`

- [ ] Add inventory, capability, version-compatibility, and environment posture for payment gateway plugins alongside existing runtime plugin evidence.
- [ ] Surface plugin ownership, rollout status, sandbox posture, and secret-readiness checks in the admin control plane.
- [ ] Ensure payment gateways, provider runtimes, and core infrastructure plugins use the same governance vocabulary instead of ad hoc status surfaces.
- [ ] Run: `cargo test -p sdkwork-api-interface-admin plugin_governance_routes -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "feat: add plugin governance center for commercial runtime"`

### Task 13: Add reconciliation, invoices, disputes, and finops reproducibility

**Files:**
- Modify: `crates/sdkwork-api-app-finance/src/lib.rs`
- Modify: `crates/sdkwork-api-app-payments/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Create: `crates/sdkwork-api-app-finance/tests/reconciliation_batches.rs`
- Create: `docs/superpowers/specs/2026-04-04-finops-reconciliation-design.md`

- [ ] Implement scheduled reconciliation batches for Stripe, Alipay, and WeChat provider evidence.
- [ ] Implement invoice and tax-document metadata plus operator export paths, even if country-specific tax calculation sophistication lands in a later slice.
- [ ] Implement dispute tracking and refund investigation views so finance can reproduce every commercial movement from immutable evidence.
- [ ] Run: `cargo test -p sdkwork-api-app-finance reconciliation_batches -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "feat: add reconciliation and finops closure"`

### Task 14: Finish PostgreSQL commercial parity and make unsupported dialect behavior explicit

**Files:**
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`
- Modify: `crates/sdkwork-api-storage-mysql/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-libsql/src/lib.rs`
- Modify: `crates/sdkwork-api-app-runtime/src/lib.rs`
- Modify: `crates/sdkwork-api-product-runtime/src/lib.rs`

- [ ] Complete PostgreSQL parity for payment, finance, async jobs, pricing, and settlement paths.
- [ ] Make runtime startup fail fast when MySQL or LibSQL are configured for unsupported commercial capabilities.
- [ ] Add compatibility reporting so the admin control plane can explain dialect posture honestly.
- [ ] Run: `cargo test -p sdkwork-api-storage-postgres -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "feat: close postgres commercial parity"`

### Task 15: Freeze resilience, performance, and commercial launch gates

**Files:**
- Create: `crates/sdkwork-api-interface-http/tests/commercial_launch_gates.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/runtime_execution.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-overview/src/view-model.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-operations/src/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/pages/index.tsx`
- Create: `docs/superpowers/specs/2026-04-04-commercial-launch-gates.md`
- Create: `docs/superpowers/specs/2026-04-04-commercial-readiness-scorecard.md`

- [ ] Add resilience tests for duplicate callbacks, duplicate recharge grants, job callback replay, provider timeout recovery, and degraded queue visibility.
- [ ] Publish launch gates for payment drift, settlement drift, reconciliation delay, async backlog, callback replay suppression, and latency budgets.
- [ ] Surface launch-gate posture in admin and tenant-safe SLO posture in portal.
- [ ] Run: `cargo test -p sdkwork-api-interface-http commercial_launch_gates -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "feat: freeze commercial launch gates"`

### Task 16: Rebaseline the whole platform and prepare execution waves

**Files:**
- Modify: `docs/superpowers/specs/2026-04-04-global-commercial-system-design.md`
- Modify: `docs/superpowers/plans/2026-04-04-global-commercial-system-program.md`
- Modify: `docs/superpowers/specs/2026-04-04-commercial-launch-gates.md`
- Modify: `docs/superpowers/specs/2026-04-04-commercial-readiness-scorecard.md`

- [ ] Re-score P0, P1, and P2 after each completed wave rather than waiting until the end.
- [ ] Keep execution waves small enough to preserve review quality: payment kernel, gateways, recharge closure, async jobs, admin, portal, reconciliation, parity, launch gates.
- [ ] Do not call the system commercially complete until all P0 items are evidenced by tests, control-plane visibility, and reconciliation reports.
- [ ] Run: `git status --short`
- [ ] Final checkpoint commit once the wave is fully verified.

### Task 17: Add the coupon and growth marketing system

**Files:**
- Create: `docs/superpowers/specs/2026-04-04-coupon-and-growth-marketing-system-design.md`
- Create: `docs/superpowers/plans/2026-04-04-coupon-and-growth-marketing-program.md`
- Modify: historical legacy coupon domain compatibility layer
- Modify: historical legacy coupon app compatibility layer
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-coupons/src/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/pages/index.tsx`

- [ ] Replace the current one-campaign-one-code model with template, campaign, batch, code, claim, and redemption semantics.
- [ ] Add secure code-pool management, one-template-many-codes support, and finance-safe redemption fulfillment.
- [ ] Add referral and invite growth architecture as part of the same marketing kernel, not as disconnected portal behavior.
- [ ] Execute the detailed workstream in `docs/superpowers/plans/2026-04-04-coupon-and-growth-marketing-program.md`.
- [ ] Checkpoint commit: `git commit -m "plan: add coupon and growth marketing workstream"`

## Execution Order

The best delivery order is:

1. freeze the boundary and add canonical payment plus finance domains
2. land storage and application kernels
3. land Stripe first, then Alipay and WeChat Pay
4. close recharge grants into canonical commercial accounts
5. complete async multimodal execution
6. sync admin and portal to the canonical commercial surfaces
7. add coupon and growth marketing foundations
8. add plugin governance and capability visibility
9. add reconciliation, invoices, disputes, and finops evidence
10. finish PostgreSQL parity
11. freeze launch gates and readiness scorecards

## Exit Criteria

This plan is complete only when all of the following are true:

- payment collection is real for Stripe, Alipay, and WeChat Pay
- payment success, finance journal, and balance grant stay consistent under replay and retry
- refunds, disputes, invoices, and reconciliation are first-class control-plane objects
- coupon templates, code pools, and redemptions are canonical and finance-safe
- async image, video, audio, and music work runs through the durable job kernel
- admin and portal both consume canonical payment, finance, pricing, and job APIs
- PostgreSQL is production-valid for the same commercial kernel as SQLite
- launch gates are explicit, tested, and visible
