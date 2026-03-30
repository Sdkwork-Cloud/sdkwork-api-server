# Router Gateway Command Center Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `Gateway` command center to `sdkwork-router-portal` that makes compatibility, deployment modes, role topology, and commerce readiness explicit inside the product.

**Architecture:** Keep the change frontend-local. Add a new bounded portal package, wire it into the shell route model, and feed the page with explicit product constants plus existing workspace posture hints so the page stays reliable without waiting on new backend APIs.

**Tech Stack:** TypeScript, React, Vite, existing portal package architecture, node:test assertions

---

### Task 1: Add failing route and product tests

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-architecture.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`

- [ ] **Step 1: Write a failing route expectation**

Add assertions that the portal route model includes `gateway`, the route path exists, and the package list includes a `sdkwork-router-portal-gateway` module.

- [ ] **Step 2: Run the focused architecture test**

Run: `node --test apps/sdkwork-router-portal/tests/portal-architecture.test.mjs`
Expected: FAIL because the new route and package do not exist yet.

- [ ] **Step 3: Write a failing product-surface expectation**

Add assertions that the new page exposes desktop mode, server mode, role topology, compatibility rows for Codex, Claude Code, Gemini-compatible clients, and OpenClaw, plus actions that point toward API Keys, Routing, and Billing.

- [ ] **Step 4: Run the focused product test**

Run: `node --test apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
Expected: FAIL because the page does not exist yet.

### Task 2: Wire the new route and bounded package

**Files:**
- Modify: `apps/sdkwork-router-portal/package.json`
- Modify: `apps/sdkwork-router-portal/tsconfig.json`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/routes.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/routePaths.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-gateway/package.json`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-gateway/src/index.tsx`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-gateway/src/types/index.ts`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-gateway/src/services/index.ts`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-gateway/src/components/index.tsx`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-gateway/src/pages/index.tsx`

- [ ] **Step 1: Extend shared route types and route metadata**

Add `gateway` to the shared route union, define `/gateway`, and add navigation copy that positions the route as the router command center.

- [ ] **Step 2: Add the bounded package entry**

Register the new workspace package in portal app dependencies and tsconfig path mapping.

- [ ] **Step 3: Lazy-load the new page in `AppRoutes`**

Mount the route with the same protected-shell pattern used by the existing portal modules.

### Task 3: Implement the page view model and UI

**Files:**
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-gateway/src/types/index.ts`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-gateway/src/services/index.ts`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-gateway/src/components/index.tsx`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-gateway/src/pages/index.tsx`

- [ ] **Step 1: Define a small page-local model**

Represent:

- compatibility rows
- runtime mode cards
- topology playbooks
- readiness actions

- [ ] **Step 2: Build static product constants plus derived readiness hints**

Use current product truths from existing docs and route names. Add light derivation from current billing/runway posture where useful.

- [ ] **Step 3: Build the page**

Render:

- gateway posture
- compatibility matrix
- mode switchboard
- topology playbooks
- readiness and commerce actions

- [ ] **Step 4: Keep navigation explicit**

Add actions that navigate to API Keys, Routing, and Billing instead of duplicating those pages.

### Task 4: Align README and docs with the new product surface

**Files:**
- Modify: `apps/sdkwork-router-portal/README.md`
- Modify: `docs/reference/api-compatibility.md`
- Modify: `docs/architecture/runtime-modes.md`

- [ ] **Step 1: Document the new route in the portal README**

Describe `Gateway` as the place where compatibility, deployment modes, and launch topology are surfaced.

- [ ] **Step 2: Update compatibility docs**

Make the agent-client compatibility story clearer and consistent with the new in-product route.

- [ ] **Step 3: Update runtime-mode docs**

Tie desktop mode and server mode back to the portal product entrypoint.

### Task 5: Verify targeted tests and portal typecheck

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-architecture.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`

- [ ] **Step 1: Run focused route and product tests**

Run:

- `node --test apps/sdkwork-router-portal/tests/portal-architecture.test.mjs`
- `node --test apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`

Expected: PASS.

- [ ] **Step 2: Run portal typecheck**

Run: `pnpm --dir apps/sdkwork-router-portal typecheck`
Expected: PASS.

- [ ] **Step 3: Run portal build if typecheck is clean**

Run: `pnpm --dir apps/sdkwork-router-portal build`
Expected: PASS.
