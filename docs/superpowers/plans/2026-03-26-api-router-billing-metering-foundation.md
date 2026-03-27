# API Router Billing And Metering Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the first production-ready foundation of router metering, account holds, settlement, and benefit-lot accounting so every OpenAI, Anthropic Messages, Gemini, and Claude Code compatible request can be admitted, metered, and billed through one canonical backend.

**Architecture:** Introduce new canonical domain and storage contracts for request metering, pricing plans, accounts, holds, benefits, and settlements. Keep existing admin and portal APIs alive through compatibility projections while the gateway moves from coarse quota checks to strong-consistency hold and settlement flows.

**Tech Stack:** Rust, sqlx, Axum, serde, SQLite, MySQL, LibSQL, React, TypeScript, pnpm workspace, node:test

---

### Task 1: Freeze new domain contracts and failing tests

**Files:**
- Modify: `crates/sdkwork-api-domain-usage/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-billing/src/lib.rs`
- Create: `crates/sdkwork-api-domain-billing/tests/request_settlement.rs`
- Create: `crates/sdkwork-api-domain-billing/tests/account_benefit_lot.rs`
- Create: `crates/sdkwork-api-domain-usage/tests/request_meter_fact.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`

- [ ] **Step 1: Add failing tests for request meter facts, request metric rows, pricing snapshots, account holds, account ledger entries, and benefit lots**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-domain-billing -p sdkwork-api-domain-usage -- --nocapture`**
- [ ] **Step 3: Add minimal domain structs and enums for the new metering and settlement kernel**
- [ ] **Step 4: Extend `AdminStore` with new methods for accounts, holds, settlements, pricing plans, and request meter facts**
- [ ] **Step 5: Re-run the same domain test command and confirm the new contracts compile and pass**

### Task 2: Add canonical schema and storage migration coverage

**Files:**
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-mysql/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-libsql/src/lib.rs`
- Create: `crates/sdkwork-api-storage-sqlite/tests/billing_metering_schema.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/sqlite_migrations.rs`

- [ ] **Step 1: Add failing migration tests asserting creation of the new pricing, account, hold, settlement, and commerce tables**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-storage-sqlite sqlite_migrations billing_metering_schema -- --nocapture`**
- [ ] **Step 3: Implement new canonical tables and compatibility views inside SQLite migrations**
- [ ] **Step 4: Mirror the same schema additions in MySQL and LibSQL stores where applicable**
- [ ] **Step 5: Re-run the targeted storage tests and confirm green**

### Task 3: Implement storage round-trip for accounts, holds, and settlements

**Files:**
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Create: `crates/sdkwork-api-storage-sqlite/tests/accounting_roundtrip.rs`

- [ ] **Step 1: Write failing SQLite round-trip tests for account creation, benefit-lot issuance, hold allocation, hold release, and settlement capture**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-storage-sqlite accounting_roundtrip -- --nocapture`**
- [ ] **Step 3: Implement the minimal store queries for the new accounting types**
- [ ] **Step 4: Add compatibility projections for legacy usage and billing summary readers**
- [ ] **Step 5: Re-run the targeted SQLite store tests and confirm green**

### Task 4: Introduce pricing plan resolution and meter normalization services

**Files:**
- Create: `crates/sdkwork-api-app-billing/src/pricing.rs`
- Create: `crates/sdkwork-api-app-billing/src/accounting.rs`
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-usage/src/lib.rs`
- Create: `crates/sdkwork-api-app-billing/tests/pricing_resolution.rs`
- Create: `crates/sdkwork-api-app-usage/tests/provider_usage_normalization.rs`

- [ ] **Step 1: Write failing tests for price-plan precedence and provider usage normalization across OpenAI, Anthropic, and Gemini**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-app-billing -p sdkwork-api-app-usage -- --nocapture`**
- [ ] **Step 3: Implement normalized meter extraction and pricing-plan matching**
- [ ] **Step 4: Implement hold estimation helpers and settlement calculation helpers**
- [ ] **Step 5: Re-run the same focused app-layer tests and confirm green**

### Task 5: Move gateway admission from quota-only to hold-and-settle

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/compat_anthropic.rs`
- Modify: `crates/sdkwork-api-interface-http/src/compat_gemini.rs`
- Create: `crates/sdkwork-api-interface-http/tests/request_hold_settlement.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/generation_billing_guardrails.rs`

- [ ] **Step 1: Add failing gateway tests for hold creation, insufficient-balance rejection, actual settlement capture, and unused-hold release**
- [ ] **Step 2: Add provider-specific tests proving OpenAI, Anthropic Messages, and Gemini paths all write normalized request meter facts**
- [ ] **Step 3: Run `cargo test -p sdkwork-api-interface-http request_hold_settlement generation_billing_guardrails anthropic_messages_route gemini_generate_content_route -- --nocapture`**
- [ ] **Step 4: Replace coarse `check_quota` admission with account hold orchestration**
- [ ] **Step 5: Persist request facts, metrics, charge lines, and settlements instead of only `usage_record + ledger_entry`**
- [ ] **Step 6: Re-run the focused gateway test command and confirm green**

### Task 6: Add admin APIs for proxy-provider billing governance

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Modify: `crates/sdkwork-api-app-catalog/src/lib.rs`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/src/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts`

- [ ] **Step 1: Add failing admin route tests for provider-model offerings, pricing plan CRUD, account views, and coupon templates**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-interface-admin sqlite_admin_routes -- --nocapture`**
- [ ] **Step 3: Implement new admin routes scoped around proxy-provider governance instead of route-page pricing edits**
- [ ] **Step 4: Extend admin TypeScript types and API clients for the new control-plane records**
- [ ] **Step 5: Re-run the targeted admin route tests and confirm green**

### Task 7: Add portal APIs for balances, orders, and settlement history

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/src/index.ts`
- Create: `crates/sdkwork-api-interface-admin/tests/portal_billing_routes.rs`

- [ ] **Step 1: Add failing portal route tests for account balance, benefit lots, request settlement list, coupon redemption, and recharge order creation**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-interface-admin portal_billing_routes -- --nocapture`**
- [ ] **Step 3: Implement authenticated portal endpoints backed by the new accounting tables**
- [ ] **Step 4: Extend portal shared types and API clients**
- [ ] **Step 5: Re-run the targeted portal route tests and confirm green**

### Task 8: Align admin and portal UI with the new backend model

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-catalog/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-apirouter/src/pages/GatewayRoutesPage.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-apirouter/src/pages/GatewayUsagePage.tsx`
- Modify: `apps/sdkwork-router-admin/tests/admin-crud-ux.test.mjs`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-account/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/tests/portal-evidence-polish.test.mjs`

- [ ] **Step 1: Add failing UI tests for provider-first pricing management in admin and real balance or settlement evidence in portal**
- [ ] **Step 2: Run `pnpm --dir apps/sdkwork-router-admin test` and `pnpm --dir apps/sdkwork-router-portal test` if the workspaces already expose those scripts, otherwise run the targeted node tests directly**
- [ ] **Step 3: Move provider pricing controls under proxy-provider governance in admin**
- [ ] **Step 4: Replace seeded portal billing and credit views with live account, lot, and settlement data**
- [ ] **Step 5: Re-run the targeted UI tests and confirm green**

### Task 9: Compatibility, migration, and documentation hardening

**Files:**
- Modify: `docs/reference/api-compatibility.md`
- Modify: `docs/zh/reference/api-compatibility.md`
- Modify: `docs/api-reference/admin-api.md`
- Modify: `docs/zh/api-reference/admin-api.md`
- Modify: `docs/api-reference/portal-api.md`
- Modify: `docs/zh/api-reference/portal-api.md`

- [ ] **Step 1: Document the canonical metering model and provider-specific usage normalization**
- [ ] **Step 2: Document that Claude Code, Anthropic Messages, OpenAI, and Gemini all settle through the same hold-and-settle kernel**
- [ ] **Step 3: Document migration behavior for `ai_usage_records`, `ai_billing_ledger_entries`, and `ai_model_price` compatibility views**
- [ ] **Step 4: Run the highest-value backend verification commands**

```bash
cargo test -p sdkwork-api-domain-billing -p sdkwork-api-domain-usage -- --nocapture
cargo test -p sdkwork-api-storage-sqlite -- --nocapture
cargo test -p sdkwork-api-interface-http anthropic_messages_route gemini_generate_content_route request_hold_settlement -- --nocapture
cargo test -p sdkwork-api-interface-admin sqlite_admin_routes portal_billing_routes -- --nocapture
cargo fmt --all
cargo check --workspace
```

- [ ] **Step 5: Run the highest-value frontend verification commands**

```bash
pnpm --dir apps/sdkwork-router-admin typecheck
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir apps/sdkwork-router-portal build
```

- [ ] **Step 6: Review `git diff` and confirm the new kernel fully replaces coarse quota-only accounting on the critical request paths**

## Follow-On Plans

This plan deliberately prioritizes the foundation. Follow-on dedicated plans should cover:

- enterprise postpaid and invoice workflows
- subscription billing provider integration
- refund, dispute, and credit-note operations
- analytics warehouse and cost or margin reporting
