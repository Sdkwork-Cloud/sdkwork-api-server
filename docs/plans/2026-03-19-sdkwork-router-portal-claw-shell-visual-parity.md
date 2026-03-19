# SDKWork Router Portal Claw Shell Visual Parity Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the Portal authenticated shell visually match `claw-studio` while preserving Portal-specific business content and routes.

**Architecture:** Keep the current Portal route and auth architecture, but refactor the shell chrome to follow the `claw-studio` titlebar, rail, and settings-shell patterns. Theme state remains store-driven through `ThemeManager`, with shell styling aligned via shared spacing, scrollbar, color-scheme, and state treatment.

**Tech Stack:** React 19, React Router 7, Zustand, Tailwind CSS 4, Radix Dialog, Tauri desktop chrome

---

### Task 1: Lock the new shell contract in tests

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`

**Step 1: Write the failing test expectations**

Update the tests so they require:

- glass header classes matching `claw-studio`
- centered workspace slot in the header
- no old `Active workspace` block in `Sidebar.tsx`
- `PORTAL_COLLAPSED_SIDEBAR_WIDTH = 72`
- `PORTAL_MIN_SIDEBAR_WIDTH = 220`
- settings sections styled like `claw-studio` settings cards
- scrollbar and dark color-scheme rules in `src/theme.css`

**Step 2: Run test to verify it fails**

Run: `node --test apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`

Expected: FAIL because the current shell still uses the old header/sidebar/config-center contracts.

**Step 3: Commit**

Do not commit yet. Continue after the red phase.

### Task 2: Rebuild the header and sidebar shell chrome

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/AppHeader.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/WindowControls.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/Sidebar.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/SidebarProfileDock.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/lib/portalPreferences.ts`

**Step 1: Implement the claw-style titlebar**

Update `AppHeader.tsx` so it mirrors the `claw-studio` shell rhythm:

- translucent glass header surface
- left-aligned brand mark
- centered workspace chip built from Portal workspace data
- right-aligned `WindowControls`

Update `WindowControls.tsx` to match the `claw-studio` hover rhythm.

**Step 2: Implement the claw-style rail**

Update `Sidebar.tsx` to:

- remove the top workspace summary block
- keep grouped navigation rows
- preserve collapse and resize behavior
- use `claw-studio` rail spacing and affordance styling

Align sidebar constants in `portalPreferences.ts` with `claw-studio`.

**Step 3: Tune the footer dock**

Adjust `SidebarProfileDock.tsx` so the trigger and popover feel like a Portal-flavored sibling of the `claw-studio` account control.

**Step 4: Run test to verify the shell contract passes**

Run: `node --test apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`

Expected: PASS

### Task 3: Rebuild the config center and theme finishing layer

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/ConfigCenter.tsx`
- Modify: `apps/sdkwork-router-portal/src/theme.css`

**Step 1: Rebuild config center sections**

Refactor `ConfigCenter.tsx` to use `claw-studio Settings`-style sections:

- search + left navigation rail
- right content pane
- `Appearance`, `Navigation`, and `Workspace` sections
- `claw-studio`-style theme mode and theme color controls
- Portal-specific shell preview retained as a subordinate section

**Step 2: Align the theme substrate**

Update `src/theme.css` to add:

- `claw-studio`-style scrollbar variables and rules
- explicit dark `color-scheme`
- any shell token adjustments needed for the new glass/header/rail parity

**Step 3: Run test to verify the settings and theme contract passes**

Run: `node --test apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs`

Expected: PASS

### Task 4: Verify the full Portal app

**Files:**
- No new files expected

**Step 1: Run focused shell tests**

Run: `node --test apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`

Expected: PASS

**Step 2: Run typecheck**

Run: `pnpm --dir apps/sdkwork-router-portal typecheck`

Expected: PASS

**Step 3: Run build**

Run: `pnpm --dir apps/sdkwork-router-portal build`

Expected: PASS

**Step 4: Commit**

Do not commit unless explicitly requested by the user.
