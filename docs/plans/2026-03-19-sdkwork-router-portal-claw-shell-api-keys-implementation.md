# SDKWork Router Portal Claw Shell API Keys Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Align Portal shell, config center, and API Key management with the claw-studio shell and api-router interaction model.

**Architecture:** Keep the existing Portal route and API contracts, but rebuild the config center and API key page around claw-studio-inspired internal shells, toolbars, dialogs, and table interactions. Prefer front-end flexibility over storage migration where backend support is already sufficient.

**Tech Stack:** React 19, React Router 7, Zustand, existing Portal commons components, Rust Portal API contracts.

---

### Task 1: Lock the new UI contract with failing tests

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs`

**Step 1: Write the failing test**

- Add assertions for:
  - settings-shell style config center navigation and search
  - API key manager toolbar/search/filter/dialog copy
  - custom environment and lifecycle policy affordances

**Step 2: Run test to verify it fails**

Run: `node --test tests/portal-theme-config.test.mjs tests/portal-product-polish.test.mjs`

Expected: FAIL because the current ConfigCenter and PortalApiKeysPage do not contain the new manager/settings shell contract.

**Step 3: Write minimal implementation**

- Implement only enough UI structure and copy to satisfy the new contract while preserving existing behavior.

**Step 4: Run test to verify it passes**

Run: `node --test tests/portal-theme-config.test.mjs tests/portal-product-polish.test.mjs`

Expected: PASS

### Task 2: Rebuild config center as a claw-style settings shell

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/ConfigCenter.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/store/usePortalShellStore.ts`

**Step 1: Write the failing test**

- Add assertions for:
  - tabbed settings shell
  - search input
  - appearance/navigation/workspace sections
  - live preview content

**Step 2: Run test to verify it fails**

Run: `node --test tests/portal-theme-config.test.mjs`

Expected: FAIL

**Step 3: Write minimal implementation**

- Replace the old two-column card layout with a left-nav/right-content settings shell.
- Add any small store helpers needed for reset/default flows.

**Step 4: Run test to verify it passes**

Run: `node --test tests/portal-theme-config.test.mjs`

Expected: PASS

### Task 3: Rebuild Portal API Key management around a manager/table/dialog flow

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/services/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/types/index.ts`
- Add: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyManagerToolbar.tsx`
- Add: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyTable.tsx`
- Add: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyDialogs.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/components/index.tsx`

**Step 1: Write the failing test**

- Add assertions for:
  - manager toolbar actions
  - search/filter controls
  - usage method dialog
  - custom environment and lifecycle policy fields

**Step 2: Run test to verify it fails**

Run: `node --test tests/portal-product-polish.test.mjs`

Expected: FAIL

**Step 3: Write minimal implementation**

- Build the manager toolbar, table, and dialogs.
- Keep repository calls unchanged.
- Preserve latest plaintext handling after key creation.

**Step 4: Run test to verify it passes**

Run: `node --test tests/portal-product-polish.test.mjs`

Expected: PASS

### Task 4: Tighten shell integration and visual consistency

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/providers/AppProviders.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/Sidebar.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/SidebarProfileDock.tsx`

**Step 1: Write the failing test**

- Assert the new shell/settings contract markers.

**Step 2: Run test to verify it fails**

Run: `node --test tests/portal-theme-config.test.mjs tests/portal-product-polish.test.mjs`

Expected: FAIL if shell markers are missing.

**Step 3: Write minimal implementation**

- Tighten slots, copy, and consistency details that keep the Portal shell visually aligned with claw-studio.

**Step 4: Run test to verify it passes**

Run: `node --test tests/portal-theme-config.test.mjs tests/portal-product-polish.test.mjs`

Expected: PASS

### Task 5: Verify the app end-to-end

**Files:**
- No direct file changes required

**Step 1: Run Portal tests**

Run: `node --test tests/portal-theme-config.test.mjs tests/portal-product-polish.test.mjs`

Expected: PASS

**Step 2: Run Portal typecheck/build**

Run: `pnpm typecheck`

Expected: PASS

Run: `pnpm build`

Expected: PASS

**Step 3: Verify final diff**

Run: `git diff -- apps/sdkwork-router-portal docs/plans`

Expected: Only intended Portal shell / API key / plan changes

