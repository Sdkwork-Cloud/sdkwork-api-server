# Router Admin Claw-Shell Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rebuild `apps/sdkwork-router-admin` into a `claw-studio`-style shell with persistent theming, a settings center, and a collapsible sidebar while preserving all live admin workflows.

**Architecture:** Introduce a new shell package and settings package, move app-shell concerns out of the current core package, replace hash routing with `react-router-dom`, and centralize theme/sidebar preferences in a persisted store. Existing business pages remain mostly intact and inherit the new visual system through shared shell/layout components and reworked commons styling.

**Tech Stack:** React 19, TypeScript, Vite, react-router-dom, Zustand, Lucide React, existing workspace packages

---

### Task 1: Document New Shell Contracts in Tests

**Files:**
- Modify: `apps/sdkwork-router-admin/tests/admin-architecture.test.mjs`

**Step 1: Write the failing test**

Add assertions for:

- `sdkwork-router-admin-shell` package presence
- `sdkwork-router-admin-settings` package presence
- route manifest includes `settings`
- shell code contains `Sidebar`, `ThemeManager`, and `BrowserRouter`
- core store exposes theme and sidebar state

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs`
Expected: FAIL because the shell/settings packages and new contracts do not exist yet.

**Step 3: Write minimal implementation**

Create the shell/settings package scaffolding and new exports just enough to satisfy the contract tests.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs`
Expected: PASS

### Task 2: Add Theme and Shell State in Core

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/src/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/package.json`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/routes.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/store.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/routePaths.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/index.tsx`

**Step 1: Write the failing test**

Extend the architecture test to assert:

- `ThemeMode`
- `ThemeColor`
- `sidebarWidth`
- `toggleSidebar`
- `hiddenSidebarItems`
- exported route paths include `/settings`

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs`
Expected: FAIL on missing store/theme exports.

**Step 3: Write minimal implementation**

Add a persisted Zustand store with:

- `themeMode`
- `themeColor`
- `isSidebarCollapsed`
- `sidebarWidth`
- `hiddenSidebarItems`

Add route-path exports and extend route definitions for shell routing.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs`
Expected: PASS

### Task 3: Build the Shell Package

**Files:**
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/package.json`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/index.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/AppRoot.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/AppProviders.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/ThemeManager.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/MainLayout.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/AppHeader.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/Sidebar.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/AppRoutes.tsx`

**Step 1: Write the failing test**

Extend the architecture test to assert the shell package exports and references:

- `BrowserRouter`
- `Sidebar`
- `ThemeManager`
- `AdminLoginPage`
- `SettingsPage`

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs`
Expected: FAIL until the new shell package is implemented.

**Step 3: Write minimal implementation**

Create the shell with:

- browser router using `/admin/`
- guarded layout
- header
- collapsible sidebar
- right-side content pane

Wire existing page components into route rendering.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs`
Expected: PASS

### Task 4: Add the Settings Package

**Files:**
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-settings/package.json`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-settings/src/index.tsx`

**Step 1: Write the failing test**

Extend the architecture test to assert the settings package contains:

- `General`
- theme options
- sidebar visibility controls
- references to all theme color ids

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs`
Expected: FAIL until the settings package exists.

**Step 3: Write minimal implementation**

Create a settings page with:

- left-side settings nav
- general tab
- theme mode controls
- theme color controls
- sidebar visibility toggles

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs`
Expected: PASS

### Task 5: Rewire the App Entry to the New Shell

**Files:**
- Modify: `apps/sdkwork-router-admin/package.json`
- Modify: `apps/sdkwork-router-admin/tsconfig.json`
- Modify: `apps/sdkwork-router-admin/src/App.tsx`

**Step 1: Write the failing test**

Extend the architecture test to require:

- root app imports the shell package instead of the old core app
- root dependencies include the new shell/settings packages
- TypeScript paths include the new packages

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs`
Expected: FAIL until root wiring is updated.

**Step 3: Write minimal implementation**

Update root app composition and workspace dependency graph to mount the new shell.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs`
Expected: PASS

### Task 6: Restyle the Shared Visual System

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-commons/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/src/theme.css`

**Step 1: Write the failing test**

Extend the architecture test to assert:

- theme file defines the six `claw-studio` theme variants
- shared UI includes shell-aware class names for cards, pills, tables, buttons, and heroes
- theme file includes sidebar collapsed-state and settings-shell selectors

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs`
Expected: FAIL until the new theme and shell styles are present.

**Step 3: Write minimal implementation**

Replace the old industrial dark CSS with a `claw-studio`-aligned token system and shell styling. Keep business-page markup stable where possible.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs`
Expected: PASS

### Task 7: Bring Login and Business Pages Into the New Visual Language

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-auth/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-overview/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-users/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-tenants/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-coupons/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-catalog/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-traffic/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-operations/src/index.tsx`

**Step 1: Write the failing test**

Extend the architecture test with strings that prove:

- login is branded as part of the new shell system
- each business page is still reachable through shell routes
- settings and shell classes are referenced where needed

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs`
Expected: FAIL until page-level shell integration is present.

**Step 3: Write minimal implementation**

Tune business pages to fit the new shell:

- tighter hero copy
- better shell-aware actions
- cleaner empty states and notes
- visual consistency with the new card/table system

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs`
Expected: PASS

### Task 8: Verify the Full Workspace

**Files:**
- Modify as needed from previous tasks

**Step 1: Run typecheck**

Run: `pnpm --dir apps/sdkwork-router-admin typecheck`
Expected: PASS

**Step 2: Run architecture tests**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-architecture.test.mjs`
Expected: PASS

**Step 3: Run production build**

Run: `pnpm --dir apps/sdkwork-router-admin build`
Expected: PASS

**Step 4: Review results and polish**

Fix any type, routing, or packaging issues that surface.
