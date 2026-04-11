# Bootstrap Update Packs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add versioned bootstrap update packs so `/data` can support baseline initialization plus ordered, idempotent incremental data updates for dev and prod deployments.

**Architecture:** Keep the existing domain-grouped `/data/*` JSON layout, but extend profile manifests to reference `/data/updates/*.json` manifests. Each update manifest carries stable update metadata, dependency ordering, and additional per-domain file lists. The loader merges baseline profile refs with ordered update refs, validates dependency integrity, and reuses existing upsert-oriented stages so repeated startup safely applies both new records and record updates. Catalog semantics stay explicit: `channel = canonical inventor`, `provider = official/proxy/local execution entry`, `provider-model = canonical-to-provider model mapping`, and `model-price = canonical model plus execution-provider price contract`.

**Tech Stack:** Rust, serde/serde_json, existing bootstrap loader/registry, repository `/data` JSON assets, Tokio tests, SQLite-backed `AdminStore`

---

### Task 1: Add failing update-pack tests

**Files:**
- Modify: `crates/sdkwork-api-app-runtime/src/tests.rs`
- Modify: `crates/sdkwork-api-product-runtime/tests/product_runtime.rs`

- [ ] **Step 1: Write failing app-runtime tests**

Cover:
- profile manifest `updates` loading in declared order
- update dependency validation failure on missing prerequisite
- record update behavior for existing provider/routing metadata through update packs

- [ ] **Step 2: Write failing product-runtime assertions**

Cover:
- repository `prod` and `dev` profiles exposing update-pack seeded catalog/routing/workspace records

- [ ] **Step 3: Run targeted tests to verify failure**

Run:
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

Expected: FAIL because profile manifests do not yet support update packs.

### Task 2: Implement update-pack manifest loading

**Files:**
- Modify: `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`
- Modify: `crates/sdkwork-api-app-runtime/src/bootstrap_data/mod.rs`

- [ ] **Step 1: Add shared bundle-ref manifest struct**

Introduce a reusable manifest payload for domain file lists so profile and update manifests stay high-cohesion and low-coupling.

- [ ] **Step 2: Add update manifest support**

Implement:
- `updates` field on profile manifests
- `BootstrapUpdateManifest` with `update_id`, `release_version`, `depends_on`, `description`, and per-domain file refs
- ordered loading and merge of update refs into the final data pack

- [ ] **Step 3: Add validation**

Validate:
- duplicate update IDs
- missing or out-of-order dependencies
- duplicate update manifest file references
- empty update IDs / malformed version metadata

- [ ] **Step 4: Expose update metadata in load outcome**

Carry loaded update IDs/version info so runtime results can report what got applied.

- [ ] **Step 5: Run targeted tests**

Run: `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
Expected: PASS

### Task 3: Add repository update packs and expanded seed data

**Files:**
- Modify: `data/profiles/prod.json`
- Modify: `data/profiles/dev.json`
- Add: `data/updates/2026-04-global-catalog-expansion.json`
- Add: `data/updates/2026-04-dev-experience-expansion.json`
- Add: `data/channels/2026-04-global-expansion.json`
- Add: `data/providers/2026-04-global-expansion.json`
- Add: `data/official-provider-configs/2026-04-global-expansion.json`
- Add: `data/models/2026-04-global-expansion.json`
- Add: `data/channel-models/2026-04-global-expansion.json`
- Add: `data/provider-models/2026-04-global-expansion.json`
- Add: `data/model-prices/2026-04-global-expansion.json`
- Add: `data/routing/2026-04-global-expansion.json`
- Add: `data/tenants/2026-04-dev-expansion.json`
- Add: `data/projects/2026-04-dev-expansion.json`
- Add: `data/api-key-groups/2026-04-dev-expansion.json`
- Add: `data/observability/2026-04-dev-expansion.json`
- Add: `data/quota-policies/2026-04-dev-expansion.json`

- [ ] **Step 1: Create production catalog expansion update**

Seed new official channels/providers/models, provider-model subsets, official/proxy/local price rows, and routing defaults that represent post-install growth without modifying baseline files in place.

- [ ] **Step 2: Create dev experience expansion update**

Seed additional demo workspace/project/API key group/quota/observability data that depends on the catalog expansion update.

- [ ] **Step 3: Wire profiles to ordered updates**

`prod` should load the global catalog expansion update.
`dev` should load both the global catalog expansion update and the dev experience expansion update.

- [ ] **Step 4: Run targeted runtime tests**

Run: `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`
Expected: PASS

### Task 4: Extend admin pricing and provider-governance contract

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/src/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-catalog/src/page/shared.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-catalog/src/page/CatalogModelPriceDialog.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-catalog/src/page/CatalogDetailPanel.tsx`
- Modify: `apps/sdkwork-router-admin/tests/admin-catalog-pricing-contract.test.mjs`

- [ ] **Step 1: Add failing admin contract test**

Cover:
- `ModelPriceTier` exposure in admin types
- model price create/save payload including `price_source_kind`, `billing_notes`, and `pricing_tiers`
- catalog detail and dialog surfacing tiered-pricing metadata

- [ ] **Step 2: Wire admin contract updates**

Implement:
- provider-governance aware model-price types
- model-price create/update payload support for friendly pricing metadata
- detail-panel display of official/proxy/local pricing posture and tiered pricing evidence
- dialog support for editing price source, billing notes, and pricing tiers JSON

- [ ] **Step 3: Run focused admin verification**

Run:
- `node --test apps/sdkwork-router-admin/tests/admin-catalog-pricing-contract.test.mjs`
- `pnpm --dir apps/sdkwork-router-admin typecheck`

Expected: PASS

### Task 5: Verify update-safe bootstrap behavior

**Files:**
- Modify if needed: `crates/sdkwork-api-app-runtime/src/tests.rs`
- Modify if needed: `crates/sdkwork-api-product-runtime/tests/product_runtime.rs`

- [ ] **Step 1: Run focused bootstrap verification**

Run:
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

Expected: PASS

- [ ] **Step 2: Run broader regression verification**

Run:
- `cargo test -p sdkwork-api-app-runtime --lib`
- `cargo test -p sdkwork-api-product-runtime`
- `pnpm --dir apps/sdkwork-router-admin typecheck`

Expected: PASS

- [ ] **Step 3: Review resulting data update contract**

Confirm the framework now supports:
- baseline bootstrap
- ordered update-pack overlays
- stable upsert-based updates
- canonical channel/provider/provider-model/model-price/route semantics
- official, proxy, and local pricing metadata with optional tier detail
- future additive seed packs without rewriting loader stages
