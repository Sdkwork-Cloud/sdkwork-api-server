# SDKWork Router Portal Wide Workspace Layout Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove global page-top shell summaries and convert Portal authenticated pages to a wider workspace-first layout.

**Architecture:** Simplify the authenticated shell by removing `ShellStatus`, then refactor each Portal page so the first viewport opens directly on work surfaces instead of summary strips and metric-card preambles. Preserve existing data-loading and page capabilities while rebalancing layout hierarchy.

**Tech Stack:** React 19, React Router 7, Tailwind 4 utilities, shared Portal commons components.

---

### Task 1: Lock the new wide-layout contract with failing tests

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs`

**Step 1: Write the failing test**

- Assert that:
  - `MainLayout` no longer renders `ShellStatus`
  - the content wrapper no longer includes `max-w-[1600px]`
  - top `portalx-status-row` usage is removed from Portal pages
  - API Keys no longer render the top credentials/status summary card

**Step 2: Run test to verify it fails**

Run: `node --test tests/portal-theme-config.test.mjs tests/portal-product-polish.test.mjs`

Expected: FAIL

**Step 3: Write minimal implementation**

- Only implement enough layout changes to satisfy the new contract.

**Step 4: Run test to verify it passes**

Run: `node --test tests/portal-theme-config.test.mjs tests/portal-product-polish.test.mjs`

Expected: PASS

### Task 2: Simplify the authenticated shell

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/app/PortalProductApp.tsx`

**Step 1: Write the failing test**

- Assert removal of `ShellStatus` wiring and the fixed-width shell content wrapper.

**Step 2: Run test to verify it fails**

Run: `node --test tests/portal-theme-config.test.mjs tests/portal-product-polish.test.mjs`

Expected: FAIL

**Step 3: Write minimal implementation**

- Remove `ShellStatus` rendering and any no-longer-needed props.
- Expand the main content wrapper to full available width.

**Step 4: Run test to verify it passes**

Run: `node --test tests/portal-theme-config.test.mjs tests/portal-product-polish.test.mjs`

Expected: PASS

### Task 3: Rebuild page openings so content starts immediately

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-routing/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-usage/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-user/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-account/src/pages/index.tsx`

**Step 1: Write the failing test**

- Assert page files do not include `portalx-status-row` in the primary layout and do not lead with metric-card summary strips.

**Step 2: Run test to verify it fails**

Run: `node --test tests/portal-product-polish.test.mjs`

Expected: FAIL

**Step 3: Write minimal implementation**

- Move important actions into relevant surfaces.
- Remove top summary strips and metric-card openings.
- Begin each page with the first real work area.

**Step 4: Run test to verify it passes**

Run: `node --test tests/portal-product-polish.test.mjs`

Expected: PASS

### Task 4: Verify the result end-to-end

**Files:**
- No direct file changes required

**Step 1: Run tests**

Run: `node --test tests/portal-theme-config.test.mjs tests/portal-product-polish.test.mjs`

Expected: PASS

**Step 2: Run typecheck and build**

Run: `pnpm typecheck`

Expected: PASS

Run: `pnpm build`

Expected: PASS

**Step 3: Run a local smoke check**

- Open the authenticated Portal shell.
- Verify that pages now open directly on work surfaces and that the right content region uses the available width more aggressively.
