# Provider Account Routing First Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first-class `provider-account` layer so one provider can own multiple executable accounts, bootstrap them from `/data`, manage them through admin CRUD, and let gateway execution select an eligible account after provider routing.

**Architecture:** Keep provider-level route selection intact and add an incremental second phase under it. `provider-account` becomes the catalog/runtime bridge: catalog stores account identity and routing hints, `extension_instance` remains the executable binding and credential owner, and gateway resolves a provider first then chooses an account under that provider, falling back to the existing provider-level path when no accounts exist.

**Tech Stack:** Rust, Axum admin API, SQLx SQLite/Postgres stores, serde JSON bootstrap packs, existing extension-instance runtime binding, React/TypeScript admin router UI for routing strategy normalization

---

### Task 1: Lock The Domain And Store Contract

**Files:**
- Modify: `crates/sdkwork-api-domain-catalog/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-core/src/admin_store.rs`
- Add: `crates/sdkwork-api-storage-sqlite/tests/provider_accounts.rs`
- Add: `crates/sdkwork-api-storage-postgres/tests/provider_accounts.rs`

- [ ] **Step 1: Write the failing storage contract tests for provider-account roundtrip**
  Run: `cargo test -p sdkwork-api-storage-sqlite provider_accounts -- --nocapture`
  Expected: FAIL because `ProviderAccountRecord` and store methods do not exist yet.

- [ ] **Step 2: Add the minimal domain type**
  Add `ProviderAccountRecord` with ids, account kind, owner scope, execution binding, base URL override, region, priority, weight, enablement, notes, and hint fields. Keep defaults additive and serde-friendly for `/data`.

- [ ] **Step 3: Extend the `AdminStore` contract**
  Add `upsert_provider_account`, `list_provider_accounts`, `find_provider_account`, and `delete_provider_account`, plus convenience filter helpers for provider-scoped account lookup.

- [ ] **Step 4: Re-run the targeted storage contract tests**
  Run: `cargo test -p sdkwork-api-storage-sqlite provider_accounts -- --nocapture`
  Expected: still FAIL in store impls/migrations, but trait and type compile.

### Task 2: Implement SQLite And Postgres Persistence

**Files:**
- Modify: `crates/sdkwork-api-storage-sqlite/src/sqlite_migration_catalog_gateway_schema.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/catalog_store.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/admin_store_impl.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/postgres_migration_catalog_gateway_schema.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/catalog_store.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/admin_store_impl.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`

- [ ] **Step 1: Write the failing persistence tests against real SQLite/Postgres adapters**
  Run: `cargo test -p sdkwork-api-storage-sqlite provider_accounts -- --nocapture`
  Run: `cargo test -p sdkwork-api-storage-postgres provider_accounts -- --nocapture`
  Expected: FAIL because the schema and SQL helpers are missing.

- [ ] **Step 2: Add additive schema for `ai_provider_account`**
  Include primary key, `provider_id`, `execution_instance_id`, account metadata, routing hints, enablement, timestamps, and provider/account lookup indexes.

- [ ] **Step 3: Implement CRUD/upsert methods in both stores**
  Reuse the existing catalog-store pattern so the `AdminStore` implementation delegates cleanly and idempotently.

- [ ] **Step 4: Re-run storage tests**
  Run: `cargo test -p sdkwork-api-storage-sqlite provider_accounts -- --nocapture`
  Run: `cargo test -p sdkwork-api-storage-postgres provider_accounts -- --nocapture`
  Expected: PASS for roundtrip and idempotent upsert/delete behavior.

### Task 3: Expose Admin CRUD For Provider Accounts

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/types.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/catalog.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/routes.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/openapi.rs`
- Add: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes/provider_accounts.rs`

- [ ] **Step 1: Write failing admin API tests for create/list/delete provider accounts**
  Run: `cargo test -p sdkwork-api-interface-admin provider_accounts -- --nocapture`
  Expected: FAIL because endpoints and request/response types do not exist.

- [ ] **Step 2: Add request/response DTOs**
  Include `provider_id`, `provider_account_id`, `display_name`, `account_kind`, `owner_scope`, `owner_tenant_id`, `execution_instance_id`, `base_url_override`, `region`, `priority`, `weight`, `enabled`, `routing_tags`, and operator hints.

- [ ] **Step 3: Implement admin handlers with validation**
  Validate provider existence and, when present, `execution_instance_id` existence. Return `400` for bad references and `404` for deletes on missing ids.

- [ ] **Step 4: Register routes and OpenAPI surface**
  Add `GET /admin/provider-accounts`, `POST /admin/provider-accounts`, and `DELETE /admin/provider-accounts/{provider_account_id}`.

- [ ] **Step 5: Re-run admin CRUD tests**
  Run: `cargo test -p sdkwork-api-interface-admin provider_accounts -- --nocapture`
  Expected: PASS with idempotent create/update and clean delete semantics.

### Task 4: Bootstrap `/data` Provider Accounts And Documentation

**Files:**
- Modify: `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`
- Modify: `crates/sdkwork-api-app-runtime/src/bootstrap_data/registry.rs`
- Add: `data/provider-accounts/default.json`
- Add: `data/provider-accounts/2026-04-official-and-proxy-accounts.json`
- Modify: `data/profiles/dev.json`
- Modify: `data/profiles/prod.json`
- Add: `docs/step/2026-04-10-provider-account-routing-first-slice-step-update.md`

- [ ] **Step 1: Write a failing bootstrap test that imports provider accounts twice**
  Run: `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
  Expected: FAIL or missing coverage because provider accounts are not part of the manifest/import order yet.

- [ ] **Step 2: Extend bootstrap manifest support**
  Add `provider_accounts` to bundle refs and `BootstrapDataPack`, with additive loading under `/data/provider-accounts/*.json`.

- [ ] **Step 3: Insert a new bootstrap stage between provider config and model stages**
  Upsert accounts after providers and runtime instances are available, preserving reference safety and idempotency.

- [ ] **Step 4: Add default data**
  Seed one default official/proxy account per major provider already present in catalog data, binding each to the provider’s existing `extension_instance` by `execution_instance_id`.

- [ ] **Step 5: Re-run bootstrap tests**
  Run: `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
  Run: `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`
  Expected: PASS with no duplicate dirty data on repeated bootstrap.

### Task 5: Select Provider Accounts During Gateway Execution

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/gateway_provider_resolution.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/gateway_types.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Add: `crates/sdkwork-api-app-gateway/tests/provider_accounts.rs`

- [ ] **Step 1: Write failing gateway tests for account selection**
  Cover: default account selected, disabled account skipped, region-preferred account wins, and fallback to provider-level execution when no accounts exist.
  Run: `cargo test -p sdkwork-api-app-gateway provider_accounts -- --nocapture`
  Expected: FAIL because gateway only resolves at provider granularity.

- [ ] **Step 2: Extend execution descriptors with provider-account context**
  Include selected `provider_account_id`, effective `execution_instance_id`, resolved base URL, and account-scoped hints without breaking existing provider-level callers.

- [ ] **Step 3: Implement account selection after provider routing**
  Resolve provider accounts for the selected provider, filter enabled accounts, prefer requested region and higher priority/weight, and use provider-level fallback when no eligible account exists.

- [ ] **Step 4: Bind execution to `extension_instance`**
  Use the selected account’s `execution_instance_id` instead of assuming `provider.id`, and keep local fallback semantics unchanged when the bound instance is disabled or unavailable.

- [ ] **Step 5: Re-run gateway tests**
  Run: `cargo test -p sdkwork-api-app-gateway provider_accounts -- --nocapture`
  Expected: PASS for account-aware execution selection and provider fallback behavior.

### Task 6: Normalize Routing Strategy Values In Admin UI

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutingProfilesDialog.tsx`
- Add: `apps/sdkwork-router-admin/tests/admin-routing-strategy-normalization.test.mjs`

- [ ] **Step 1: Write the failing frontend contract test**
  Assert that the create-routing-profile dialog emits canonical backend values and does not default to legacy strings like `priority`.
  Run: `node --test apps/sdkwork-router-admin/tests/admin-routing-strategy-normalization.test.mjs`
  Expected: FAIL because the dialog still seeds legacy values.

- [ ] **Step 2: Normalize UI defaults and options**
  Default to `deterministic_priority`, keep displayed labels friendly, and only submit backend-supported enum strings.

- [ ] **Step 3: Re-run the targeted frontend test**
  Run: `node --test apps/sdkwork-router-admin/tests/admin-routing-strategy-normalization.test.mjs`
  Expected: PASS.

### Task 7: Final Verification

**Files:**
- No code changes unless verification reveals a defect

- [ ] **Step 1: Run storage verification**
  Run: `cargo test -p sdkwork-api-storage-sqlite provider_accounts -- --nocapture`

- [ ] **Step 2: Run admin verification**
  Run: `cargo test -p sdkwork-api-interface-admin provider_accounts -- --nocapture`

- [ ] **Step 3: Run gateway verification**
  Run: `cargo test -p sdkwork-api-app-gateway provider_accounts -- --nocapture`

- [ ] **Step 4: Run bootstrap verification**
  Run: `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
  Run: `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

- [ ] **Step 5: Run frontend verification**
  Run: `node --test apps/sdkwork-router-admin/tests/admin-routing-strategy-normalization.test.mjs`

- [ ] **Step 6: If failures remain, fix the smallest valid cause and re-run only the impacted checks, then finish with one full targeted verification sweep**
