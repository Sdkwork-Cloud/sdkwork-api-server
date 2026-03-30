# Aggregated Cloud Gateway Account System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild `sdkwork-api-router` onto a professional aggregated cloud-gateway account kernel with `ai_`-prefixed canonical tables, user-scoped payable accounts, tenant-scoped reporting, unified JWT and API-key attribution, lot-based balances, hold-then-settle execution, and versioned meter and pricing rules.

**Architecture:** Land the system in layers. First freeze the account, identity, and settlement contracts around `tenant_id BIGINT`, `organization_id BIGINT DEFAULT 0`, and `user_id BIGINT`. Then add canonical storage, projection compatibility, gateway auth context, row-based usage metrics, pricing-plan resolution, and admin or portal governance over the new kernel.

**Tech Stack:** Rust, sqlx, Axum, serde, SQLite, Postgres, TypeScript, React, pnpm workspace, node:test

---

### Task 1: Freeze canonical IDs, subjects, and account-domain contracts

**Files:**
- Modify: `crates/sdkwork-api-domain-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-usage/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-commerce/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Create: `crates/sdkwork-api-domain-billing/tests/account_kernel.rs`
- Create: `crates/sdkwork-api-domain-billing/tests/account_hold.rs`
- Create: `crates/sdkwork-api-domain-usage/tests/request_meter_fact.rs`
- Create: `crates/sdkwork-api-domain-identity/tests/gateway_auth_subject.rs`

- [ ] **Step 1: Add failing domain tests for user-scoped accounts, benefit lots, holds, ledger entries, request facts, and auth subjects**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-domain-billing -p sdkwork-api-domain-usage -p sdkwork-api-domain-identity -- --nocapture`**
- [ ] **Step 3: Replace string-only billing ownership contracts with canonical `tenant_id`, `organization_id`, and `user_id` bigint-oriented records**
- [ ] **Step 4: Extend `AdminStore` with canonical methods for accounts, lots, holds, request facts, pricing plans, and settlements**
- [ ] **Step 5: Re-run the same focused domain test command and confirm the contracts compile and pass**

### Task 2: Add canonical `ai_` schema and compatibility projections

**Files:**
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Create: `crates/sdkwork-api-storage-sqlite/tests/account_schema.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/sqlite_migrations.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`

- [ ] **Step 1: Write failing migration tests asserting creation of the new canonical `ai_` account, hold, lot, pricing, request, and settlement tables**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-storage-sqlite sqlite_migrations account_schema -- --nocapture`**
- [ ] **Step 3: Implement SQLite schema additions with `tenant_id BIGINT`, `organization_id BIGINT NOT NULL DEFAULT 0`, and immutable event-table rules**
- [ ] **Step 4: Mirror the schema changes in Postgres and keep compatibility projections for `ai_usage_records`, `ai_billing_ledger_entries`, and `ai_model_price`**
- [ ] **Step 5: Re-run the focused storage tests for SQLite and Postgres and confirm green**

### Task 3: Implement unified gateway auth context for Java `PlusAuthToken` JWT and API keys

**Files:**
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Create: `crates/sdkwork-api-app-identity/tests/plus_auth_token_gateway_context.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/gateway_auth_context.rs`

- [ ] **Step 1: Add failing tests for resolving `tenant_id + organization_id + user_id` from Java-compatible JWT claims**
- [ ] **Step 2: Add failing tests for resolving the same subject from API key ownership plus nullable `api_key_id` behavior for JWT requests**
- [ ] **Step 3: Run `cargo test -p sdkwork-api-app-identity -p sdkwork-api-interface-http gateway_auth_context plus_auth_token_gateway_context -- --nocapture`**
- [ ] **Step 4: Implement canonical gateway auth context parsing, secret resolution hooks, and ownership validation**
- [ ] **Step 5: Re-run the focused auth-context tests and confirm green**

### Task 4: Implement lot-based balances, holds, and immutable ledger flows

**Files:**
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`
- Create: `crates/sdkwork-api-app-billing/src/accounting.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Create: `crates/sdkwork-api-storage-sqlite/tests/accounting_roundtrip.rs`

- [ ] **Step 1: Add failing tests for account creation, benefit-lot issuance, hold allocation, hold release, settlement capture, and balance projections**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-app-billing -p sdkwork-api-storage-sqlite accounting_roundtrip -- --nocapture`**
- [ ] **Step 3: Implement the account-lot allocator with the standard spend priority: expiry, narrow scope, promo or package, then cash**
- [ ] **Step 4: Implement immutable ledger entries and lot-level allocation evidence for each account mutation**
- [ ] **Step 5: Re-run the focused accounting tests and confirm green**

### Task 5: Add canonical meter definitions and versioned pricing plans

**Files:**
- Modify: `crates/sdkwork-api-domain-usage/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-billing/src/lib.rs`
- Create: `crates/sdkwork-api-app-billing/src/pricing.rs`
- Modify: `crates/sdkwork-api-app-usage/src/lib.rs`
- Create: `crates/sdkwork-api-app-billing/tests/pricing_resolution.rs`
- Create: `crates/sdkwork-api-app-usage/tests/provider_usage_normalization.rs`

- [ ] **Step 1: Write failing tests for canonical metrics such as `token.input`, `token.output`, `token.cache_read`, `image.count`, and `audio.second`**
- [ ] **Step 2: Add failing tests for pricing-plan precedence and pricing snapshots on request settlement**
- [ ] **Step 3: Run `cargo test -p sdkwork-api-app-billing -p sdkwork-api-app-usage pricing_resolution provider_usage_normalization -- --nocapture`**
- [ ] **Step 4: Implement row-based meter normalization and versioned price-plan selection**
- [ ] **Step 5: Re-run the focused pricing and normalization tests and confirm green**

### Task 6: Move gateway execution from quota-only billing to hold-and-settle

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/compat_anthropic.rs`
- Modify: `crates/sdkwork-api-interface-http/src/compat_gemini.rs`
- Create: `crates/sdkwork-api-interface-http/tests/request_hold_settlement.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/generation_billing_guardrails.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/anthropic_messages_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/gemini_generate_content_route.rs`

- [ ] **Step 1: Add failing gateway tests for hold creation, insufficient balance rejection, actual settlement capture, release of unused holds, and per-API-key attribution**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-interface-http request_hold_settlement generation_billing_guardrails anthropic_messages_route gemini_generate_content_route -- --nocapture`**
- [ ] **Step 3: Replace coarse quota-only admission with canonical request facts, holds, metrics, charge lines, and settlements**
- [ ] **Step 4: Ensure every request writes `tenant_id`, `organization_id`, `user_id`, `auth_type`, and `api_key_id(nullable)` to the request fact**
- [ ] **Step 5: Re-run the focused gateway settlement tests and confirm green**

### Task 7: Add admin and portal governance over the new account kernel

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Create: `crates/sdkwork-api-interface-portal/tests/account_kernel_routes.rs`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/src/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/src/index.ts`

- [ ] **Step 1: Add failing route tests for account summaries, benefit lots, holds, settlements, API-key attribution, and tenant aggregate reporting**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-interface-admin sqlite_admin_routes -- --nocapture` and `cargo test -p sdkwork-api-interface-portal account_kernel_routes -- --nocapture`**
- [ ] **Step 3: Implement admin routes for governance of users, API keys, accounts, plans, and settlements**
- [ ] **Step 4: Implement portal routes for user-facing balances, lots, settlement history, and API-key breakdowns**
- [ ] **Step 5: Re-run the focused admin and portal route tests and confirm green**

### Task 8: Upgrade admin and portal products to the canonical account system

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/workbench.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/src/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-account/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-account/src/repository/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/repository/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`

- [ ] **Step 1: Add failing UI tests for user-account balances, lot evidence, settlement lines, and API-key usage attribution**
- [ ] **Step 2: Run `pnpm --dir apps/sdkwork-router-portal test` and the admin workspace tests already present in the repo**
- [ ] **Step 3: Replace summary-only billing views with live account, lot, hold, and settlement data**
- [ ] **Step 4: Surface tenant aggregates without turning tenant into a payable wallet**
- [ ] **Step 5: Re-run the targeted frontend tests and confirm green**

### Task 9: Hardening, documentation, migration, and rollout

**Files:**
- Modify: `docs/reference/api-compatibility.md`
- Modify: `docs/zh/reference/api-compatibility.md`
- Modify: `docs/api-reference/admin-api.md`
- Modify: `docs/zh/api-reference/admin-api.md`
- Modify: `docs/api-reference/portal-api.md`
- Modify: `docs/zh/api-reference/portal-api.md`
- Modify: `README.md`
- Modify: `README.zh-CN.md`

- [ ] **Step 1: Document the canonical `ai_` account system, bigint identity rules, and `organization_id DEFAULT 0` policy**
- [ ] **Step 2: Document the unified JWT and API-key request-subject model and the migration away from quota-only accounting**
- [ ] **Step 3: Document compatibility projections for `ai_usage_records`, `ai_billing_ledger_entries`, and `ai_model_price`**
- [ ] **Step 4: Run the highest-value backend verification commands**

```bash
cargo test -p sdkwork-api-domain-billing -p sdkwork-api-domain-usage -p sdkwork-api-domain-identity -- --nocapture
cargo test -p sdkwork-api-storage-sqlite -p sdkwork-api-storage-postgres -- --nocapture
cargo test -p sdkwork-api-app-identity -p sdkwork-api-app-billing -p sdkwork-api-app-usage -- --nocapture
cargo test -p sdkwork-api-interface-http request_hold_settlement gateway_auth_context anthropic_messages_route gemini_generate_content_route -- --nocapture
cargo test -p sdkwork-api-interface-admin sqlite_admin_routes -- --nocapture
cargo test -p sdkwork-api-interface-portal account_kernel_routes -- --nocapture
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

- [ ] **Step 6: Review `git diff` and confirm the critical request paths no longer depend on quota-only accounting**

## Rollout Notes

- keep the old summary and coarse pricing tables as compatibility projections until all consumers migrate
- prefer dual-write and projection rollout before cutting over the gateway execution path
- do not mix tenant reporting scopes with user payable scopes
- do not allow destructive edits to ledger and settlement evidence

## Follow-On Plans

After this plan lands, create dedicated follow-on plans for:

- invoice and finance-export integration
- tax and jurisdictional charging
- enterprise postpaid accounts and credit limits
- anomaly detection and budgets
- FOCUS-aligned analytics export
