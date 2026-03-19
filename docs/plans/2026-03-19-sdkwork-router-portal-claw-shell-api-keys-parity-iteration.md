# SDKWork Router Portal Claw Shell API Keys Parity Iteration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Eliminate the remaining visual and product-structure gap between `sdkwork-router-portal` and `claw-studio` for the shell settings experience, sidebar behavior, and API Key management.

**Architecture:** Keep Portal routing and backend contracts unchanged, but replace Portal-specific page storytelling with much more literal adaptations of the `claw-studio` `Settings`, `ApiRouter`, and `UnifiedApiKey*` shells. Favor matching layout hierarchy, spacing, surface treatment, and interaction order over preserving earlier custom cards and dashboards.

**Tech Stack:** React 19, React Router 7, Zustand, Tailwind 4 utilities, existing Portal commons/dialog primitives, Portal API contracts.

---

### Task 1: Lock tighter claw parity with failing tests

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`

**Step 1: Write the failing test**

- Add assertions for literal claw-inspired shell markers:
  - `ConfigCenter` outer layout, left rail, search shell, nav button classes, right content wrapper
  - API Key page manager/table/dialog class structure and removal of Portal-only metrics/tabs storytelling

**Step 2: Run test to verify it fails**

Run: `node --test tests/portal-theme-config.test.mjs tests/portal-product-polish.test.mjs`

Expected: FAIL because the current Portal implementation still contains custom product structure and does not include the tighter claw-style shells.

**Step 3: Write minimal implementation**

- Implement only the structural and content changes needed to satisfy the new parity contract while keeping existing data flows intact.

**Step 4: Run test to verify it passes**

Run: `node --test tests/portal-theme-config.test.mjs tests/portal-product-polish.test.mjs`

Expected: PASS

### Task 2: Rebuild `ConfigCenter` into a more literal `Settings` adaptation

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/ConfigCenter.tsx`

**Step 1: Write the failing test**

- Assert presence of:
  - `flex h-full bg-zinc-50/50 dark:bg-zinc-950/50`
  - `flex w-72 shrink-0 flex-col border-r border-zinc-200 bg-zinc-50/80 backdrop-blur-xl`
  - `scrollbar-hide flex-1 overflow-x-hidden overflow-y-auto`
  - `mx-auto w-full max-w-5xl p-8 md:p-12`

**Step 2: Run test to verify it fails**

Run: `node --test tests/portal-theme-config.test.mjs`

Expected: FAIL

**Step 3: Write minimal implementation**

- Remove the custom pill-heavy dialog narrative.
- Port the Claw settings shell hierarchy into the modal body.
- Keep Portal-specific theme and workspace controls inside Claw-like cards and spacing.

**Step 4: Run test to verify it passes**

Run: `node --test tests/portal-theme-config.test.mjs`

Expected: PASS

### Task 3: Replace the Portal API Key page with a literal manager/table/dialog workflow

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyManagerToolbar.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyTable.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyDialogs.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/services/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/types/index.ts`

**Step 1: Write the failing test**

- Assert presence of:
  - `data-slot="api-router-page"`
  - `data-slot="portal-api-key-manager"`
  - Claw-style rounded zinc surfaces for toolbar/table/dialogs
  - removal of Portal-only `Rotation checklist`, `Environment strategy`, and metrics grid storytelling

**Step 2: Run test to verify it fails**

Run: `node --test tests/portal-product-polish.test.mjs`

Expected: FAIL

**Step 3: Write minimal implementation**

- Collapse the page into a Claw-like manager shell.
- Keep flexible create flows:
  - environment presets
  - custom environment
  - lifecycle presets
  - one-time plaintext handling
- Re-map Portal data into a table structure that visually matches Claw even where backend fields differ.

**Step 4: Run test to verify it passes**

Run: `node --test tests/portal-product-polish.test.mjs`

Expected: PASS

### Task 4: Tighten sidebar/shell visual parity where it still drifts

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/Sidebar.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/SidebarProfileDock.tsx`

**Step 1: Write the failing test**

- Add assertions only if needed for missing shell markers after Tasks 2-3 land.

**Step 2: Run test to verify it fails**

Run: `node --test tests/portal-theme-config.test.mjs tests/portal-product-polish.test.mjs`

Expected: FAIL only if the sidebar shell still diverges on the checked markers.

**Step 3: Write minimal implementation**

- Align navigation density, label treatment, and bottom dock behavior more closely with Claw while preserving Portal theme tokens and collapse/resize support.

**Step 4: Run test to verify it passes**

Run: `node --test tests/portal-theme-config.test.mjs tests/portal-product-polish.test.mjs`

Expected: PASS

### Task 5: Verify code and visual outcome

**Files:**
- No direct file changes required

**Step 1: Run Portal tests**

Run: `node --test tests/portal-theme-config.test.mjs tests/portal-product-polish.test.mjs`

Expected: PASS

**Step 2: Run typecheck/build**

Run: `pnpm typecheck`

Expected: PASS

Run: `pnpm build`

Expected: PASS

**Step 3: Run a visual smoke check**

Run the Portal app locally, capture API Key and Config Center screenshots, and compare them against `claw-studio` reference pages.

Expected: Layout hierarchy, sidebar behavior, and manager/config center surfaces now read as the same product family.
