# SDKWork Router Portal Claw Dashboard And Theme Parity Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rework the Portal dashboard, shell details, config center, and theme contract so they align much more closely with claw-studio while preserving Portal route ownership and business behavior.

**Architecture:** Keep the existing Portal route/runtime architecture, but replace the dashboard information hierarchy and refine shell/config surfaces so they consume a tighter claw-style visual contract. Reuse the existing theme store and Portal data services rather than introducing a second shell system.

**Tech Stack:** React 19, React Router 7, Tailwind CSS 4, Zustand, Recharts, Radix UI, Vite, Node test runner

---

### Task 1: Lock the target with failing shell and dashboard tests

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-dashboard-analytics.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs`
- Test: `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`

**Step 1: Write the failing test**

Add assertions for:

- claw-style dashboard summary cards
- claw-style dashboard section headers and workbench layout
- config center shell preview and navigation wording staying aligned
- dashboard no longer being limited to the older operational overview contract

**Step 2: Run test to verify it fails**

Run: `node --test tests/portal-product-polish.test.mjs tests/portal-dashboard-analytics.test.mjs tests/portal-theme-config.test.mjs`

Expected: FAIL because the current dashboard and config structures do not yet satisfy the new parity contract.

**Step 3: Write minimal implementation**

Do not implement here. Move to Tasks 2-4 once the tests fail for the expected reasons.

**Step 4: Run test to verify it passes**

Run after implementation tasks complete:
`node --test tests/portal-product-polish.test.mjs tests/portal-dashboard-analytics.test.mjs tests/portal-theme-config.test.mjs`

Expected: PASS

**Step 5: Commit**

Skip commit in this workspace unless explicitly requested, because the repository already contains unrelated in-flight edits.

### Task 2: Rebuild the dashboard into a claw-style analytics workbench

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/components/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/types/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/services/index.ts`

**Step 1: Write the failing test**

Use Task 1 tests to cover:

- summary-card band
- chart workbench surfaces
- tabbed/segmented activity surface
- routing/module posture rendered in claw-style sections

**Step 2: Run test to verify it fails**

Run: `node --test tests/portal-product-polish.test.mjs tests/portal-dashboard-analytics.test.mjs`

Expected: FAIL on missing dashboard structure strings and layout contracts.

**Step 3: Write minimal implementation**

Implement:

- portal-local summary-card and section-header components styled like claw-studio
- a top KPI band driven by existing Portal metrics
- claw-style chart sections for traffic, spend, provider share, and model demand
- a lower workbench section for requests, routing evidence, quick actions, and modules

**Step 4: Run test to verify it passes**

Run: `node --test tests/portal-product-polish.test.mjs tests/portal-dashboard-analytics.test.mjs`

Expected: PASS

**Step 5: Commit**

Skip commit in this shared dirty workspace unless explicitly requested.

### Task 3: Refine config center and shell details for parity

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/ConfigCenter.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/AppHeader.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/Sidebar.tsx`

**Step 1: Write the failing test**

Use Task 1 tests to cover:

- claw-style settings navigation and content section language
- shell preview consistency
- sidebar interaction strings and parity details

**Step 2: Run test to verify it fails**

Run: `node --test tests/portal-theme-config.test.mjs tests/portal-shell-parity.test.mjs`

Expected: FAIL if the config center and shell still drift from the target contract.

**Step 3: Write minimal implementation**

Adjust:

- config center sections, wording, preview, and stat cards
- header spacing and workspace context treatment
- sidebar labels, edge affordances, and visual parity details

**Step 4: Run test to verify it passes**

Run: `node --test tests/portal-theme-config.test.mjs tests/portal-shell-parity.test.mjs`

Expected: PASS

**Step 5: Commit**

Skip commit in this shared dirty workspace unless explicitly requested.

### Task 4: Tighten shared theme and surface tokens

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- Modify: `apps/sdkwork-router-portal/src/theme.css`

**Step 1: Write the failing test**

Rely on Task 1 theme tests to enforce:

- shared token usage across dashboard, shell, and config center
- claw-style surface density and contrast treatment

**Step 2: Run test to verify it fails**

Run: `node --test tests/portal-theme-config.test.mjs`

Expected: FAIL if shared surfaces and tokens do not yet match the new contract.

**Step 3: Write minimal implementation**

Adjust:

- dashboard surface utilities
- config surface tokens
- any missing contrast, border, or hover states required for parity

**Step 4: Run test to verify it passes**

Run: `node --test tests/portal-theme-config.test.mjs`

Expected: PASS

**Step 5: Commit**

Skip commit in this shared dirty workspace unless explicitly requested.

### Task 5: Run full verification for the Portal app

**Files:**
- Test: `apps/sdkwork-router-portal/tests/*.test.mjs`

**Step 1: Write the failing test**

No new tests here. Use the updated suite as the verification target.

**Step 2: Run test to verify it fails**

Only if a regression is discovered during integration.

**Step 3: Write minimal implementation**

Fix any regression introduced by Tasks 2-4.

**Step 4: Run test to verify it passes**

Run:

- `node --test tests/portal-product-polish.test.mjs tests/portal-dashboard-analytics.test.mjs tests/portal-shell-parity.test.mjs tests/portal-theme-config.test.mjs`
- `pnpm typecheck`
- `pnpm build`

Expected: all commands exit successfully.

**Step 5: Commit**

Skip commit in this shared dirty workspace unless explicitly requested.
