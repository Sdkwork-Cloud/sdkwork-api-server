# SDKWork Router Portal Routing And Portal Upgrade Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a professional user-facing routing experience to `sdkwork-router-portal`, separate `user` from `account`, and align the portal with the real gateway and routing capabilities already present in `sdkwork-api-server`.

**Architecture:** Keep operator routing policy in the admin control plane, add a project-scoped portal routing-preferences read model for portal users, expose portal-safe routing APIs, and add new portal business packages for `routing` and `user` while repurposing `account` to the financial account domain. Keep the root app as the composition layer and let business logic stay inside packages.

**Tech Stack:** Rust, Axum, sqlx, React 19, TypeScript, Vite, pnpm, Turbo

---

## Chunk 1: Backend Contracts And Storage

### Task 1: Add failing domain and storage tests for portal routing preferences

**Files:**
- Create: `crates/sdkwork-api-storage-sqlite/tests/portal_routing_preferences.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-routing/src/lib.rs`

- [ ] **Step 1: Write failing storage tests for create and read**
- [ ] **Step 2: Run the new storage tests and verify they fail for missing trait methods or schema**
- [ ] **Step 3: Add the new routing-preference aggregate and store trait methods**
- [ ] **Step 4: Add SQLite and Postgres persistence plus schema**
- [ ] **Step 5: Re-run the storage tests and verify they pass**

### Task 2: Add failing portal interface tests for routing endpoints

**Files:**
- Create: `crates/sdkwork-api-interface-portal/tests/portal_routing.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`

- [ ] **Step 1: Write failing endpoint tests for summary, preferences, preview, and decision logs**
- [ ] **Step 2: Run the portal routing tests and verify they fail**
- [ ] **Step 3: Add portal-safe routing DTOs and handlers**
- [ ] **Step 4: Scope all routing responses to the authenticated project**
- [ ] **Step 5: Re-run the portal routing tests and verify they pass**

## Chunk 2: Frontend Package And Route Expansion

### Task 3: Add failing portal architecture tests for `routing` and `user`

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-architecture.test.mjs`
- Create: `apps/sdkwork-router-portal/tests/portal-routing-polish.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`

- [ ] **Step 1: Write failing portal tests that require new packages and navigation copy**
- [ ] **Step 2: Run the portal tests and verify they fail**
- [ ] **Step 3: Add package manifests and package skeletons for `sdkwork-router-portal-routing` and `sdkwork-router-portal-user`**
- [ ] **Step 4: Update root dependencies and route types**
- [ ] **Step 5: Re-run the portal tests and verify only the new rendering gaps remain**

### Task 4: Implement `sdkwork-router-portal-routing`

**Files:**
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-routing/package.json`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-routing/src/index.tsx`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-routing/src/types/index.ts`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-routing/src/components/index.tsx`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-routing/src/repository/index.ts`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-routing/src/services/index.ts`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-routing/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/src/index.ts`

- [ ] **Step 1: Add failing module-level expectations through the portal rendering tests**
- [ ] **Step 2: Implement portal API client functions and routing types**
- [ ] **Step 3: Implement routing services and view-model builders**
- [ ] **Step 4: Implement the routing page with summary, presets, preview, and evidence**
- [ ] **Step 5: Run portal tests and verify the routing module passes**

### Task 5: Implement `sdkwork-router-portal-user` and repurpose `account`

**Files:**
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-user/package.json`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-user/src/index.tsx`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-user/src/types/index.ts`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-user/src/components/index.tsx`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-user/src/repository/index.ts`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-user/src/services/index.ts`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-user/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-account/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-account/src/services/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-account/src/types/index.ts`

- [ ] **Step 1: Move profile and password ownership into `user`**
- [ ] **Step 2: Rewrite `account` as a financial account overview backed by billing summary plus ledger**
- [ ] **Step 3: Update tests to validate the new module semantics**
- [ ] **Step 4: Run portal tests and verify user/account separation**

## Chunk 3: Shell And Product Integration

### Task 6: Update shell composition and Overview narrative

**Files:**
- Modify: `apps/sdkwork-router-portal/src/App.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/routes.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/src/theme.css`
- Modify: `apps/sdkwork-router-portal/package.json`

- [ ] **Step 1: Add new route keys and shell navigation items**
- [ ] **Step 2: Surface routing posture inside the shell and Overview**
- [ ] **Step 3: Keep root `src` as the app composition owner as much as the current codebase allows**
- [ ] **Step 4: Ensure every module points to a meaningful next move**
- [ ] **Step 5: Run portal tests and typecheck**

## Chunk 4: Verification

### Task 7: Verify backend and frontend end to end

**Files:**
- Modify: any touched files as needed

- [ ] **Step 1: Run portal interface tests for auth, dashboard, API keys, and routing**
- [ ] **Step 2: Run routing app tests**
- [ ] **Step 3: Run portal package tests**
- [ ] **Step 4: Run portal TypeScript typecheck**
- [ ] **Step 5: Report exact verification evidence and any residual gaps**
