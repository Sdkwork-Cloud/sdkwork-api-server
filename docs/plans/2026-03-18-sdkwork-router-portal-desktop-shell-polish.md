# SDKWork Router Portal Desktop Shell Polish Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rebuild the portal shell header, sidebar footer, and shell theme wiring so the desktop experience matches the `claw-studio` standard more closely.

**Architecture:** Keep the existing portal route shell, but simplify `AppHeader` into a desktop titlebar, move shell settings behind a new sidebar profile dock, and tighten shell theming around the shared `--portal-*` token contract. The work stays inside the portal app and is protected with shell parity and theme regression tests.

**Tech Stack:** React, TypeScript, Tailwind utility classes, shared CSS custom properties in `src/theme.css`, `node:test`

---

### Task 1: Lock The New Header And Footer Product Contract In Tests

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs`

**Step 1: Write the failing tests**

Add assertions for:

- header keeps only brand identity and `WindowControls`
- header no longer exposes `Workspace shell`, `Config center`, or `Active workspace`
- sidebar footer is implemented through a dedicated profile dock component
- settings are accessed from the profile dock instead of a direct footer icon row

**Step 2: Run test to verify it fails**

Run: `node --test apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs`

Expected: FAIL because the current header still contains repeated config entry points and the current sidebar still renders direct footer buttons.

**Step 3: Commit**

Do not commit yet. Continue to the implementation tasks after the red test is confirmed.

### Task 2: Simplify The Desktop Header

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/AppHeader.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx`

**Step 1: Implement the minimal header contract**

- remove the repeated shell config buttons and centered workspace trigger from `AppHeader`
- keep only the left-aligned brand block and right-aligned `WindowControls`
- keep Tauri drag-region behavior intact
- remove any now-unused props from `MainLayout`

**Step 2: Run targeted tests**

Run: `node --test apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs`

Expected: header-related assertions pass, sidebar-footer assertions still fail until the next task is complete.

### Task 3: Introduce A Dedicated Sidebar Profile Dock

**Files:**
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/SidebarProfileDock.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/Sidebar.tsx`

**Step 1: Write the minimal shell component**

Implement `SidebarProfileDock` so it:

- renders avatar, identity, and workspace context
- supports expanded and collapsed sidebar states
- opens shell actions for config center and sign out
- replaces the old bottom icon strip in `Sidebar`

**Step 2: Keep behavior shell-safe**

- preserve `onOpenConfigCenter` and `onLogout`
- make collapsed mode show a single avatar trigger
- avoid adding new business behavior outside shell actions

**Step 3: Run targeted tests**

Run: `node --test apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs`

Expected: sidebar footer parity assertions pass.

### Task 4: Tighten The Shell Theme Hierarchy

**Files:**
- Modify: `apps/sdkwork-router-portal/src/theme.css`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/AppHeader.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/Sidebar.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/SidebarProfileDock.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/ConfigCenter.tsx`

**Step 1: Refine token-driven shell surfaces**

- ensure header, sidebar, profile dock, and config center use the same shell token hierarchy
- avoid hardcoded shell chrome colors where semantic tokens already exist
- improve the sidebar footer contrast and depth so the bottom region feels deliberate

**Step 2: Verify theme regressions**

Run: `node --test apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs`

Expected: PASS with no theme regressions.

### Task 5: Polish Integration And Remove Dead Shell State

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/AppHeader.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/Sidebar.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx`
- Modify any shell test files touched earlier

**Step 1: Remove leftover copy and imports**

- delete unused theme labels or helper buttons from the header
- remove any dead imports, props, or helper code left behind by the shell refactor
- keep the shell API minimal and readable

**Step 2: Run the full portal verification suite**

Run: `node --test apps/sdkwork-router-portal/tests/*.mjs`

Expected: PASS with zero failures.

### Task 6: Verify Typecheck, Build, And Runtime

**Files:**
- No code changes expected unless verification reveals a regression

**Step 1: Run typecheck**

Run: `pnpm --dir apps/sdkwork-router-portal typecheck`

Expected: PASS

**Step 2: Run production build**

Run: `pnpm --dir apps/sdkwork-router-portal build`

Expected: PASS

**Step 3: Run runtime shell verification**

Verify in the running preview that:

- the header shows only brand left and window controls right
- the sidebar footer shows a single profile dock
- dark and alternate theme colors still affect shell chrome and auth surfaces

**Step 4: Commit**

```bash
git add docs/plans/2026-03-18-sdkwork-router-portal-desktop-shell-polish-design.md docs/plans/2026-03-18-sdkwork-router-portal-desktop-shell-polish.md apps/sdkwork-router-portal/src/theme.css apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/AppHeader.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/Sidebar.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/SidebarProfileDock.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs
git commit -m "feat: polish portal desktop shell"
```
