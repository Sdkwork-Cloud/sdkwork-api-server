# AI Router Schema Standardization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild sdkwork-api-router onto canonical `ai_*` tables, add channel-model-provider pricing management, persist `raw_key` for app API keys, and upgrade the admin app to manage the new catalog workflow.

**Architecture:** Keep runtime/store interfaces stable where possible, introduce new domain and admin DTOs for channel models and model pricing, and move physical persistence to canonical `ai_*` tables with compatibility views for legacy names. The admin UI becomes channel-first while gateway/runtime behavior continues to consume synthesized provider-scoped model variants.

**Tech Stack:** Rust, sqlx, Axum, React, TypeScript, pnpm workspace, node:test

---

### Task 1: Canonical schema and migration tests

**Files:**
- Create: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/crates/sdkwork-api-storage-sqlite/tests/ai_schema_standardization.rs`
- Modify: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/crates/sdkwork-api-storage-sqlite/tests/sqlite_migrations.rs`
- Modify: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`

- [ ] **Step 1: Write failing migration tests for canonical `ai_*` tables and legacy compatibility**
- [ ] **Step 2: Run the targeted storage tests and verify the new assertions fail for missing tables**
- [ ] **Step 3: Implement the minimal migration changes in SQLite/Postgres**
- [ ] **Step 4: Re-run the targeted storage tests and confirm they pass**

### Task 2: Domain and storage-core extensions

**Files:**
- Modify: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/crates/sdkwork-api-domain-catalog/src/lib.rs`
- Modify: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/crates/sdkwork-api-domain-identity/src/lib.rs`
- Modify: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/crates/sdkwork-api-storage-core/src/lib.rs`

- [ ] **Step 1: Write failing tests for `raw_key`, channel models, and model price records**
- [ ] **Step 2: Run the targeted domain/storage tests and confirm they fail for missing fields or methods**
- [ ] **Step 3: Add new domain records and storage trait methods while preserving existing model-variant methods**
- [ ] **Step 4: Re-run the targeted tests and confirm the new contracts compile and pass**

### Task 3: SQLite and Postgres canonical storage implementation

**Files:**
- Modify: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/crates/sdkwork-api-storage-sqlite/tests/catalog_bindings.rs`

- [ ] **Step 1: Write failing store tests for channel-model CRUD, model-price CRUD, provider-variant synthesis, and `raw_key` persistence**
- [ ] **Step 2: Run the targeted store tests and capture expected failures**
- [ ] **Step 3: Implement canonical storage queries against `ai_*` tables plus compatibility views**
- [ ] **Step 4: Re-run the targeted store tests and confirm green**

### Task 4: App services and admin HTTP contract

**Files:**
- Modify: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/crates/sdkwork-api-app-catalog/src/lib.rs`
- Modify: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/crates/sdkwork-api-app-identity/src/lib.rs`
- Modify: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

- [ ] **Step 1: Write failing admin route tests for default channels, channel-model CRUD, model-price CRUD, and `raw_key` list visibility**
- [ ] **Step 2: Run the targeted admin route tests and confirm they fail for missing endpoints or fields**
- [ ] **Step 3: Implement canonical admin routes and legacy model-route adapters**
- [ ] **Step 4: Re-run the admin route tests and confirm green**

### Task 5: Admin type layer and client bindings

**Files:**
- Modify: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/src/index.ts`
- Modify: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts`
- Modify: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/workbench.tsx`

- [ ] **Step 1: Write failing UI-layer tests for new snapshot records and API clients**
- [ ] **Step 2: Run the admin app tests and confirm the missing symbols fail as expected**
- [ ] **Step 3: Add channel-model and model-price types, client methods, and workbench loaders/handlers**
- [ ] **Step 4: Re-run the admin app tests and confirm green**

### Task 6: Catalog UI channel-first redesign

**Files:**
- Modify: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/apps/sdkwork-router-admin/packages/sdkwork-router-admin-catalog/src/index.tsx`
- Modify: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/apps/sdkwork-router-admin/tests/admin-crud-ux.test.mjs`

- [ ] **Step 1: Write failing UI tests for channel model management and model pricing dialogs**
- [ ] **Step 2: Run the admin CRUD UX tests and confirm the new assertions fail**
- [ ] **Step 3: Implement the channel table, model list modal, and model pricing CRUD modal**
- [ ] **Step 4: Re-run the admin CRUD UX tests and confirm green**

### Task 7: Broad verification and documentation refresh

**Files:**
- Modify: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/docs/api-reference/admin-api.md`
- Modify: `D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router-local-check/docs/zh/api-reference/admin-api.md`

- [ ] **Step 1: Update API docs to describe canonical routes and schema**
- [ ] **Step 2: Run focused Rust tests for storage/admin plus admin app node tests**
- [ ] **Step 3: Run broader verification commands for the affected workspace slices**
- [ ] **Step 4: Summarize the final table layout, compatibility strategy, and current `Unified ApiKey` / router credential persistence**
