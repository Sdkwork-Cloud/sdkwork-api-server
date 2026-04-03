# Commercial System Closure Program Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** close the remaining gaps between the current `sdkwork-api-router` implementation and an advanced commercial API router platform, without regressing the plugin-first architecture or the package-based admin and portal products.

**Architecture:** preserve the current modular core and finish the missing commercial kernels in the right order: transaction-safe settlement first, gateway cutover second, control-plane completion third, async media jobs fourth, and platform-governance plus multi-database parity last. Do not widen product surfaces further until the commercial command path is correct.

**Tech Stack:** Rust, Axum, sqlx, SQLite, PostgreSQL, React, TypeScript, pnpm, cargo test

---

### Task 1: Add a public transaction seam for canonical account commands

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Create: `crates/sdkwork-api-app-billing/tests/account_kernel_mutations.rs`

- [ ] **Step 1: Add a storage-core transaction abstraction dedicated to account-kernel command execution**
- [ ] **Step 2: Write failing mutation tests for hold creation, release, and settlement idempotency**
- [ ] **Step 3: Implement SQLite transaction-backed execution for account-kernel commands**
- [ ] **Step 4: Implement PostgreSQL transaction-backed execution for the same contract**
- [ ] **Step 5: Run `cargo test -p sdkwork-api-app-billing --test account_kernel_mutations -- --nocapture`**

### Task 2: Implement canonical hold, ledger, and settlement orchestration

**Files:**
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-usage/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/account_kernel_roundtrip.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`

- [ ] **Step 1: Add app-billing command APIs for create hold, capture hold, and release hold**
- [ ] **Step 2: Persist hold allocations, ledger allocations, request facts, and request settlements through the transaction seam**
- [ ] **Step 3: Add idempotency anchors for repeated request settlement attempts**
- [ ] **Step 4: Add regression tests for insufficient balance, retry, duplicate settlement, and late correction**
- [ ] **Step 5: Run `cargo test -p sdkwork-api-app-billing -- --nocapture`**

### Task 3: Cut the HTTP gateway from legacy quota admission to canonical settlement admission

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/gateway_auth_context.rs`
- Create: `crates/sdkwork-api-interface-http/tests/canonical_account_admission.rs`

- [ ] **Step 1: Add a failing gateway test that authenticates a canonical API key and resolves its payable account**
- [ ] **Step 2: Replace legacy quota-only admission on the commercial path with canonical subject resolution, account resolution, hold planning, and settlement orchestration**
- [ ] **Step 3: Keep compatibility-era quota helpers only where the canonical account kernel is intentionally not in scope**
- [ ] **Step 4: Re-run gateway auth and admission tests**
- [ ] **Step 5: Run `cargo test -p sdkwork-api-interface-http -- --nocapture`**

### Task 4: Expose canonical commercial control-plane APIs

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Create: `crates/sdkwork-api-interface-admin/tests/account_billing_routes.rs`
- Create: `crates/sdkwork-api-interface-portal/tests/account_billing_routes.rs`
- Modify: `docs/api-reference/admin-api.md`
- Modify: `docs/api-reference/portal-api.md`

- [ ] **Step 1: Add failing admin and portal route tests for account balances, holds, benefit lots, pricing plans, pricing rates, and settlements**
- [ ] **Step 2: Implement admin routes for commercial governance and operator investigation**
- [ ] **Step 3: Implement portal routes for tenant-facing account, recharge, settlement, and pricing posture**
- [ ] **Step 4: Re-run the new admin and portal route tests**
- [ ] **Step 5: Update API reference docs**

### Task 5: Cut admin and portal packages onto the canonical backend surfaces

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-apirouter/src/pages/GatewayUsagePage.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayAccessPage.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-commercial/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/routeManifest.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/src/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-account/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/routeManifest.ts`

- [ ] **Step 1: Add typed admin and portal SDK methods for canonical account and settlement objects**
- [ ] **Step 2: Replace compatibility-era account summaries in admin and portal pages with canonical control-plane responses**
- [ ] **Step 3: Add new product modules for settlement explorer and commercial account operations**
- [ ] **Step 4: Run the existing package architecture and product tests**
- [ ] **Step 5: Run `pnpm --dir apps/sdkwork-router-admin test` and `pnpm --dir apps/sdkwork-router-portal test` where available**

### Task 6: Add a durable async multimodal job kernel

**Files:**
- Modify: `crates/sdkwork-api-domain-billing/src/lib.rs`
- Create: `crates/sdkwork-api-domain-jobs/src/lib.rs`
- Create: `crates/sdkwork-api-app-jobs/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Create: `crates/sdkwork-api-interface-admin/tests/async_jobs_routes.rs`
- Create: `crates/sdkwork-api-interface-portal/tests/async_jobs_routes.rs`

- [ ] **Step 1: Define canonical job, attempt, asset, and provider-callback records**
- [ ] **Step 2: Add storage seams and schema for durable async jobs and generated assets**
- [ ] **Step 3: Add app-job orchestration for enqueue, claim, retry, callback reconcile, and finalize**
- [ ] **Step 4: Wire long-running video, image, audio, and music paths to the job kernel**
- [ ] **Step 5: Expose admin and portal job workbench APIs**

### Task 7: Finish plugin inventory and backend product-module governance

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-app-runtime/src/lib.rs`
- Create: `crates/sdkwork-api-app-platform/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-operations/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/routeManifest.ts`

- [ ] **Step 1: Add canonical plugin and product-module inventory records**
- [ ] **Step 2: Persist compatibility snapshots, health state, and config revision evidence**
- [ ] **Step 3: Add module-level feature flags and staged rollout contracts**
- [ ] **Step 4: Expose operator inventory and compatibility APIs**
- [ ] **Step 5: Add admin workbench surfaces for plugin and module governance**

### Task 8: Finish database portability in commercial paths

**Files:**
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-mysql/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-libsql/src/lib.rs`
- Modify: `crates/sdkwork-api-app-runtime/src/lib.rs`
- Modify: `crates/sdkwork-api-product-runtime/src/lib.rs`

- [ ] **Step 1: Complete PostgreSQL canonical account and identity CRUD parity**
- [ ] **Step 2: Add MySQL storage driver beyond dialect-name placeholder behavior**
- [ ] **Step 3: Add LibSQL storage driver beyond dialect-name placeholder behavior**
- [ ] **Step 4: Verify runtime driver selection and honest unsupported fallbacks**
- [ ] **Step 5: Run driver and runtime test suites**

### Task 9: Add reconciliation, finance export, and enterprise governance polish

**Files:**
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`
- Create: `crates/sdkwork-api-app-finops/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-traffic/src/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`

- [ ] **Step 1: Add reconciliation settlement flows for late usage, stream-final usage, and duplicate callback suppression**
- [ ] **Step 2: Add finance-export and profitability projections**
- [ ] **Step 3: Add operator-facing audit export and settlement-reconciliation views**
- [ ] **Step 4: Add policy attachment points for abuse, residency, and compliance plugins**
- [ ] **Step 5: Re-run commercial billing and control-plane verification suites**

### Task 10: Rebaseline the platform and freeze the commercial v1 target

**Files:**
- Modify: `docs/superpowers/specs/2026-04-03-router-implementation-audit-and-upgrade-plan.md`
- Modify: `docs/superpowers/specs/2026-04-03-commercial-system-gap-assessment-and-target-solution.md`
- Create: `docs/superpowers/specs/2026-04-03-commercial-v1-readiness-scorecard.md`

- [ ] **Step 1: Update the audit baseline after each major phase**
- [ ] **Step 2: Add a readiness scorecard covering correctness, control plane, multimodal jobs, plugin governance, and dialect parity**
- [ ] **Step 3: Define the exact commercial-v1 exit criteria**
- [ ] **Step 4: Run the final verification matrix across backend, admin, and portal**
- [ ] **Step 5: Commit each phase separately so rollback and review remain clean**
