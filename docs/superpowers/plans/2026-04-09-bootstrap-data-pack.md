# Bootstrap Data Pack Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace thin hardcoded startup seeding with a repository-backed `/data` JSON bootstrap system that loads richer dev and production defaults for catalog, routing, commerce, billing, tenant, and marketing data.

**Architecture:** Extend `StandaloneConfig` with bootstrap data root/profile settings, add a typed importer in app runtime startup, and ship curated `/data` profile manifests plus grouped JSON bundles. The importer applies bundles in a fixed order and validates parent-child references before insert/upsert operations.

**Tech Stack:** Rust, serde/serde_json, existing `sdkwork_api_*` domain/app/store crates, Axum runtime tests, Node-free repository data assets

---

### Task 1: Config Surface

**Files:**
- Modify: `crates/sdkwork-api-config/src/types.rs`
- Modify: `crates/sdkwork-api-config/src/env_keys.rs`
- Modify: `crates/sdkwork-api-config/src/loader.rs`
- Modify: `crates/sdkwork-api-config/src/standalone_config.rs`
- Test: `crates/sdkwork-api-config/tests/config_loading.rs`

- [ ] **Step 1: Write failing config tests for bootstrap data dir/profile**
- [ ] **Step 2: Run config tests to verify missing fields fail expectations**
- [ ] **Step 3: Add config fields, env keys, file parsing, env export, reload metadata**
- [ ] **Step 4: Run config tests and fix parsing/export behavior**

### Task 2: Runtime Importer Skeleton

**Files:**
- Add: `crates/sdkwork-api-app-runtime/src/bootstrap_data.rs`
- Modify: `crates/sdkwork-api-app-runtime/src/lib.rs`
- Modify: `crates/sdkwork-api-app-runtime/src/runtime_builders.rs`
- Test: `crates/sdkwork-api-app-runtime/src/tests.rs`

- [ ] **Step 1: Write failing runtime tests for profile-driven bootstrap loading**
- [ ] **Step 2: Run app-runtime tests to confirm importer is absent**
- [ ] **Step 3: Add typed importer entrypoints, profile loading, bundle ordering, reference validation helpers**
- [ ] **Step 4: Wire importer into admin store bootstrap path after migrations**
- [ ] **Step 5: Run app-runtime tests and fix bootstrap sequencing**

### Task 3: Data Bundle Types And Domain Writers

**Files:**
- Add: `crates/sdkwork-api-app-runtime/src/bootstrap_data.rs`
- Test: `crates/sdkwork-api-app-runtime/src/tests.rs`

- [ ] **Step 1: Add typed JSON bundle structs for catalog/routing/tenant/billing/commerce/marketing**
- [ ] **Step 2: Implement idempotent apply functions per stage using existing app/store helpers**
- [ ] **Step 3: Add duplicate-ID and missing-reference validation**
- [ ] **Step 4: Re-run runtime tests and keep importer behavior deterministic**

### Task 4: Repository Data Pack

**Files:**
- Add: `data/profiles/dev.json`
- Add: `data/profiles/prod.json`
- Add: `data/channels/default.json`
- Add: `data/providers/default.json`
- Add: `data/official-provider-configs/default.json`
- Add: `data/models/default.json`
- Add: `data/channel-models/default.json`
- Add: `data/model-prices/default.json`
- Add: `data/tenants/default.json`
- Add: `data/projects/default.json`
- Add: `data/api-key-groups/default.json`
- Add: `data/routing/default.json`
- Add: `data/quota-policies/default.json`
- Add: `data/pricing/default.json`
- Add: `data/payment-methods/default.json`
- Add: `data/marketing/default.json`

- [ ] **Step 1: Create profile manifests for dev and prod**
- [ ] **Step 2: Add ranked official channels/providers and ecosystem providers**
- [ ] **Step 3: Add curated model, channel-model, and model-price bundles**
- [ ] **Step 4: Add default tenant/project/API key group/routing/pricing/payment/marketing bundles**
- [ ] **Step 5: Validate bundle references through tests**

### Task 5: Product Runtime Coverage

**Files:**
- Modify: `crates/sdkwork-api-product-runtime/tests/product_runtime.rs`
- Test: `crates/sdkwork-api-product-runtime/tests/product_runtime.rs`

- [ ] **Step 1: Add failing product runtime test asserting default bootstrap data is visible after startup**
- [ ] **Step 2: Run targeted product runtime test to confirm missing data**
- [ ] **Step 3: Adjust runtime startup integration or defaults as needed**
- [ ] **Step 4: Re-run targeted runtime tests**

### Task 6: Verification

**Files:**
- No code changes unless failures require fixes

- [ ] **Step 1: Run `cargo test -p sdkwork-api-config`**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-app-runtime`**
- [ ] **Step 3: Run `cargo test -p sdkwork-api-product-runtime`**
- [ ] **Step 4: If failures appear, fix the minimal cause and re-run affected tests**
