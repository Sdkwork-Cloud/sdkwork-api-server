# Router Admin Shell Polish Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refine `apps/sdkwork-router-admin` so its shell, sidebar, and settings center feel substantially closer to `claw-studio` without changing admin business flows.

**Architecture:** Keep the current shell/settings package split, add sidebar account-control behavior, tighten header and canvas hierarchy, and introduce a richer settings nav summary plus stage framing backed by the existing theme/sidebar store.

**Tech Stack:** React 19, TypeScript, react-router-dom, Zustand, Motion, CSS

---

### Task 1: Lock The Polish Targets In Tests

**Files:**
- Modify: `apps/sdkwork-router-admin/tests/admin-shell-parity.test.mjs`

**Step 1: Write the failing test**

Require:

- sidebar account-control popover state and animated active affordance markers
- settings nav summary and detail stage selectors

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-shell-parity.test.mjs`
Expected: FAIL until the new shell and settings polish features exist.

### Task 2: Refine Sidebar And Header Interaction Surfaces

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/components/AppHeader.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/components/Sidebar.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/styles/index.css`

**Step 1: Implement the new shell affordances**

Add:

- claw-style active indicator motion
- account control popover behavior
- bottom secondary settings action
- tighter header hierarchy and shell metadata framing

**Step 2: Verify the shell-parity test**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-shell-parity.test.mjs`
Expected: PASS for the new sidebar and header-related assertions.

### Task 3: Refine Settings Navigation And Stage Presentation

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-settings/src/Settings.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-settings/src/AppearanceSettings.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-settings/src/NavigationSettings.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-settings/src/WorkspaceSettings.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-settings/src/Shared.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/styles/index.css`

**Step 1: Implement the settings-stage polish**

Add:

- live nav summary card
- panel stage framing
- richer preview copy
- more tactile nav buttons and cards

**Step 2: Verify the shell-parity test**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-shell-parity.test.mjs`
Expected: PASS for settings summary and stage assertions.

### Task 4: Full Regression Verification

**Files:**
- Modify any files required by failing verification

**Step 1: Run all admin tests**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/*.mjs`
Expected: PASS

**Step 2: Run typecheck**

Run: `pnpm --dir apps/sdkwork-router-admin typecheck`
Expected: PASS

**Step 3: Run build**

Run: `pnpm --dir apps/sdkwork-router-admin build`
Expected: PASS
