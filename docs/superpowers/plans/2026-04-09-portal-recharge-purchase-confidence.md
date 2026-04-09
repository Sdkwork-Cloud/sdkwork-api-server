# Portal Recharge Purchase Confidence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** strengthen purchase confidence on the portal recharge page through better option guidance, a stronger checkout-style quote panel, and a mobile sticky CTA while keeping existing recharge behavior.

**Architecture:** Keep all existing repository and service seams intact, add only lightweight merchandising helpers in the recharge service layer, and refine the page composition to express stronger commercial intent. The UI contract should be locked with page-level tests before the page implementation changes.

**Tech Stack:** React 19, TypeScript, existing portal commons components, Node test runner, Vite

---

### Task 1: Lock the New UI Contract

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`

- [ ] **Step 1: Write the failing test expectations**

Add assertions for:

- `data-slot="portal-recharge-guidance-band"`
- `data-slot="portal-recharge-selection-story"`
- `data-slot="portal-recharge-quote-breakdown"`
- `data-slot="portal-recharge-mobile-cta"`
- `Best fit for steady usage`
- `Selection story`
- `Checkout summary`
- `Create order in billing`

- [ ] **Step 2: Run tests to verify they fail**

Run: `node --test tests/portal-recharge-center.test.mjs tests/portal-recharge-workflow-polish.test.mjs`

Expected: FAIL because the new slot names and purchase-confidence copy do not exist yet.

- [ ] **Step 3: Keep failures focused on the missing UI contract**

If the tests fail for an unrelated syntax or environment reason, fix the test shape and rerun until the failures clearly point to missing UI content.

### Task 2: Add Lightweight Merchandising Helpers

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/services/index.ts`
- Test: `apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`

- [ ] **Step 1: Write a failing behavior test for purchase-guidance helpers**

Add a test that verifies a helper can classify recharge options into a simple buying-intent label set using recommendation flags and relative order.

- [ ] **Step 2: Run the focused test to verify it fails**

Run: `node --test tests/portal-recharge-workflow-polish.test.mjs`

Expected: FAIL because the helper does not exist yet.

- [ ] **Step 3: Implement the minimal helper logic**

Add a focused helper that derives option guidance such as safe default, quick coverage, or reserve-building without changing API contracts.

- [ ] **Step 4: Re-run the focused test**

Run: `node --test tests/portal-recharge-workflow-polish.test.mjs`

Expected: PASS.

### Task 3: Refine the Recharge Page Composition

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx`
- Reference: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/services/index.ts`

- [ ] **Step 1: Add the guidance band and selection story blocks**

Render an always-visible guidance band near the purchase surface and a selection-story block tied to the currently selected option or custom amount.

- [ ] **Step 2: Upgrade the quote panel into a checkout summary**

Add clearer breakdown rows and copy hierarchy while preserving the existing CTA and quote-loading behavior.

- [ ] **Step 3: Add the mobile sticky CTA bar**

Show a bottom action bar only when a valid selection exists. Keep it hidden on larger breakpoints where the sticky side card already solves the action-visibility problem.

- [ ] **Step 4: Keep behavior unchanged**

Do not alter repository calls, order-creation semantics, validation rules, or history pagination logic.

### Task 4: Verify the Full Portal Recharge Surface

**Files:**
- Verify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx`
- Verify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/services/index.ts`
- Verify: `apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs`
- Verify: `apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`
- Verify: `apps/sdkwork-router-portal/tests/portal-recharge-finance-projection.test.mjs`
- Verify: `apps/sdkwork-router-portal/tests/portal-runtime-dependency-governance.test.mjs`

- [ ] **Step 1: Run the page-contract test suite**

Run: `node --test tests/portal-recharge-center.test.mjs tests/portal-recharge-workflow-polish.test.mjs tests/portal-recharge-finance-projection.test.mjs tests/portal-runtime-dependency-governance.test.mjs`

Expected: PASS.

- [ ] **Step 2: Run typecheck**

Run: `pnpm typecheck`

Expected: PASS.

- [ ] **Step 3: Run production build**

Run: `pnpm build`

Expected: PASS.

- [ ] **Step 4: Inspect git status**

Run: `git status --short --branch`

Expected: only intentional recharge-page work is present in the clean worktree.
