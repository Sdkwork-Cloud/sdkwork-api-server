# Catalog Pricing Admin Readiness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Tighten channel-model/provider-model/model-price guardrails, improve admin management for provider-supported model subsets and pricing coverage, and ship additive `/data` bootstrap updates for catalog pricing readiness.

**Architecture:** Keep the existing canonical catalog contract: `channel` owns inventor identity, `channel-model` owns canonical publication, `provider` owns executable upstream entry, `provider-model` owns provider-supported subset mapping, and `model-price` owns provider-scoped reference pricing. Strengthen backend validation so price rows cannot exist without provider support, expose provider-model and pricing coverage directly in admin, and extend `/data` through additive update packs rather than rewriting baselines.

**Tech Stack:** Rust, Axum admin API, serde JSON bootstrap packs, React/TypeScript admin console, existing sdkwork catalog/runtime/storage crates

---

### Task 1: Backend Catalog Guardrails

**Files:**
- Modify: `crates/sdkwork-api-app-catalog/Cargo.toml`
- Modify: `crates/sdkwork-api-app-catalog/src/lib.rs`
- Add: `crates/sdkwork-api-app-catalog/tests/model_price_guardrails.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/catalog.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes/providers_models_coupons.rs`

- [ ] **Step 1: Run the existing red tests for provider-model and model-price linkage**
- [ ] **Step 2: Add missing test dependency/support so catalog tests compile**
- [ ] **Step 3: Require provider existence, channel-model existence, and provider-model existence before saving model-price rows**
- [ ] **Step 4: Make provider-scoped model creation materialize provider-model and default model-price together**
- [ ] **Step 5: Map validation failures on admin create handlers to `400 Bad Request` instead of generic `500`**
- [ ] **Step 6: Re-run targeted Rust tests until the red cases go green**

### Task 2: Admin Provider-Model And Pricing Coverage

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/providerCatalog.ts`
- Add: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/providerCatalog.test.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-catalog/src/page/shared.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-catalog/src/page/useCatalogWorkspaceState.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-catalog/src/page/CatalogProviderDialog.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-catalog/src/page/CatalogDetailPanel.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-catalog/src/page/CatalogRegistrySection.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-catalog/src/page/CatalogModelPriceDialog.tsx`
- Add: `apps/sdkwork-router-admin/tests/admin-catalog-pricing-contract.test.mjs`

- [ ] **Step 1: Add failing coverage for provider supported-model serialization and pricing-readiness helpers if gaps remain**
- [ ] **Step 2: Extend provider draft helpers with pricing coverage summaries and provider-model defaults that match canonical publications**
- [ ] **Step 3: Upgrade the provider dialog so selected canonical models expose provider-model id/family, routing posture, capability flags, and token limits inline**
- [ ] **Step 4: Upgrade provider detail and registry views to surface supported-model counts, missing-price warnings, and quick pricing actions**
- [ ] **Step 5: Tighten the pricing dialog so provider choices are constrained to provider-supported models and the operator sees friendly pricing context**
- [ ] **Step 6: Run targeted frontend contract tests or package tests covering the new admin behavior**

### Task 3: Additive Bootstrap Pricing Pack And Documentation

**Files:**
- Modify: `docs/superpowers/specs/2026-04-09-bootstrap-data-pack-design.md`
- Add: `docs/step/2026-04-10-catalog-pricing-admin-readiness-step-update.md`
- Add: `data/updates/2026-04-global-catalog-pricing-admin-readiness.json`
- Add: `data/model-prices/2026-04-global-catalog-pricing-admin-readiness.json`
- Add: `data/provider-models/2026-04-global-catalog-pricing-admin-readiness.json`
- Add: `data/routing/2026-04-global-catalog-pricing-admin-readiness.json`
- Modify: `data/profiles/dev.json`
- Modify: `data/profiles/prod.json`

- [ ] **Step 1: Document the stronger provider-model-before-price rule and the admin/catalog ownership model**
- [ ] **Step 2: Add an ordered update manifest for pricing/admin readiness without mutating baseline bundles**
- [ ] **Step 3: Enrich provider-model and model-price packs with friendly billing notes, tier metadata, and route-aligned defaults**
- [ ] **Step 4: Wire the new additive pack into both dev and prod profiles**
- [ ] **Step 5: Validate bundle consistency so canonical models, provider subsets, route defaults, and price rows remain aligned**

### Task 4: Verification

**Files:**
- No code changes unless failures require fixes

- [ ] **Step 1: Run `cargo test -p sdkwork-api-app-catalog model_price_guardrails -- --nocapture`**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-interface-admin create_model_price_requires_provider_model_support -- --nocapture`**
- [ ] **Step 3: Run `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`**
- [ ] **Step 4: Run `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`**
- [ ] **Step 5: Run targeted admin package tests for provider catalog helpers and catalog UX if available**
- [ ] **Step 6: If any verification fails, fix the minimal cause and re-run the affected checks**
