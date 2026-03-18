# Router Admin Shell And Settings Parity Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refactor `apps/sdkwork-router-admin` so its shell and settings-center architecture align with `claw-studio` while preserving all live admin workflows and the current claw-like visual language.

**Architecture:** Restructure `sdkwork-router-admin-shell` into `application / components / styles`, split `sdkwork-router-admin-settings` into a multi-file settings center with query-param tabs, move shell styling ownership into the shell package, and align root bootstrap with the claw shell integration pattern.

**Tech Stack:** React 19, TypeScript, Vite, react-router-dom, Zustand, Lucide React, Motion

---

### Task 1: Lock The New Shell And Settings Structure In Tests

**Files:**
- Modify: `apps/sdkwork-router-admin/tests/admin-architecture.test.mjs`
- Modify: `apps/sdkwork-router-admin/tests/admin-shell-parity.test.mjs`
- Modify: `apps/sdkwork-router-admin/tests/admin-product-experience.test.mjs`

**Step 1: Write the failing tests**

Add assertions that require:

- shell package files under `application/app`, `application/bootstrap`, `application/layouts`, `application/providers`, `application/router`, `components`, and `styles`
- settings package files `Settings.tsx`, `GeneralSettings.tsx`, `AppearanceSettings.tsx`, `NavigationSettings.tsx`, `WorkspaceSettings.tsx`, and `Shared.tsx`
- root app bootstrap through `bootstrapShellRuntime`
- shell index imports `./styles/index.css`
- settings uses `useSearchParams`

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs tests/admin-shell-parity.test.mjs tests/admin-product-experience.test.mjs`
Expected: FAIL because the package structure is still flat.

**Step 3: Do not implement yet**

Stop after confirming the expected failures.

**Step 4: Commit**

Do not commit unless the user asks for commits.

### Task 2: Align The Root Bootstrap With The Claw Shell Pattern

**Files:**
- Modify: `apps/sdkwork-router-admin/src/App.tsx`
- Modify: `apps/sdkwork-router-admin/src/main.tsx`
- Remove or stop using: `apps/sdkwork-router-admin/src/theme.css`

**Step 1: Write the failing test**

Require:

- `App.tsx` only imports `AppRoot` from `sdkwork-router-admin-shell`
- `main.tsx` calls `bootstrapShellRuntime`
- root app no longer owns shell CSS directly

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs`
Expected: FAIL on old bootstrap and style ownership.

**Step 3: Write minimal implementation**

Move the integration model to:

- `App.tsx` renders `AppRoot`
- `main.tsx` awaits `bootstrapShellRuntime()` before mounting
- shell package owns style import

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs`
Expected: PASS

### Task 3: Restructure The Shell Package

**Files:**
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/application/app/AppRoot.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/application/bootstrap/bootstrapShellRuntime.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/application/layouts/MainLayout.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/application/providers/AppProviders.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/application/providers/ThemeManager.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/application/router/routePaths.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/components/AppHeader.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/components/Sidebar.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/components/ShellStatus.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/styles/index.css`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/index.ts`

**Step 1: Write the failing test**

Assert the new file layout and exports exist.

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs tests/admin-shell-parity.test.mjs`
Expected: FAIL until the new shell files and exports exist.

**Step 3: Write minimal implementation**

Restructure the shell without changing behavior:

- move providers, routes, and layout into `application/*`
- move header and sidebar into `components/*`
- move visual tokens into `styles/index.css`
- keep sidebar collapse, resize, and right-canvas rendering working

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs tests/admin-shell-parity.test.mjs`
Expected: PASS

### Task 4: Split The Settings Center Into Claw-Style Sections

**Files:**
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-settings/src/Settings.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-settings/src/GeneralSettings.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-settings/src/AppearanceSettings.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-settings/src/NavigationSettings.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-settings/src/WorkspaceSettings.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-settings/src/Shared.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-settings/src/index.tsx`

**Step 1: Write the failing test**

Require:

- a `Settings` composition file
- tab routing via `useSearchParams`
- split section files
- appearance, navigation, and workspace sections rendered independently

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs tests/admin-shell-parity.test.mjs`
Expected: FAIL while settings is still single-file.

**Step 3: Write minimal implementation**

Build a split settings center that keeps current admin shell preferences but follows claw structure:

- searchable left nav
- tab query param
- right detail panel
- live theme and sidebar previews

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs tests/admin-shell-parity.test.mjs`
Expected: PASS

### Task 5: Regress Business-Surface Integration

**Files:**
- Modify as needed from previous tasks
- Verify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-auth/src/index.tsx`
- Verify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-overview/src/index.tsx`
- Verify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-users/src/index.tsx`
- Verify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-tenants/src/index.tsx`
- Verify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-coupons/src/index.tsx`
- Verify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-catalog/src/index.tsx`
- Verify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-traffic/src/index.tsx`
- Verify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-operations/src/index.tsx`

**Step 1: Run the product-experience tests**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-product-experience.test.mjs`
Expected: PASS or fail only on shell wording drift introduced by the refactor.

**Step 2: Fix any product-surface regressions**

Adjust only what is required so pages still feel native inside the new shell and settings-center architecture.

**Step 3: Re-run the product-experience tests**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-product-experience.test.mjs`
Expected: PASS

### Task 6: Full Verification

**Files:**
- Modify any remaining files required by failing verification

**Step 1: Run all admin tests**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/*.mjs`
Expected: PASS

**Step 2: Run typecheck**

Run: `pnpm --dir apps/sdkwork-router-admin typecheck`
Expected: PASS

**Step 3: Run production build**

Run: `pnpm --dir apps/sdkwork-router-admin build`
Expected: PASS

**Step 4: Re-check parity-sensitive details**

Confirm:

- sidebar collapse and resize still work
- theme mode and theme color still persist
- settings tab navigation works
- right side stays the content canvas
- root bootstrap and shell package ownership now match the claw pattern

**Step 5: Commit**

Do not commit unless the user asks for commits.
