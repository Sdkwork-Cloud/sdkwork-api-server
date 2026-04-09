# Portal Recharge Settlement Continuity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** strengthen the recharge-to-billing handoff by surfacing a pending-settlement next-step callout near the checkout summary.

**Architecture:** Reuse the existing order list already loaded on the recharge page, derive a focused pending-payment spotlight in the recharge service layer, and render a new callout in the quote area without changing backend behavior or history-table responsibilities.

**Tech Stack:** React 19, TypeScript, Node test runner, existing portal commons UI

---

### Task 1: Lock the New Pending-Settlement Contract

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`

- [ ] **Step 1: Add failing assertions for the new callout**

Add assertions for:

- `data-slot="portal-recharge-next-step-callout"`
- `Pending settlement queue`
- `Latest pending order`
- `Open billing to complete payment`

- [ ] **Step 2: Run the focused tests to verify they fail**

Run: `node --test tests/portal-recharge-center.test.mjs tests/portal-recharge-workflow-polish.test.mjs`

Expected: FAIL because the new callout contract is not implemented yet.

### Task 2: Add a Pending-Settlement Spotlight Helper

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/services/index.ts`
- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`

- [ ] **Step 1: Add a failing helper test**

Write a test proving the helper picks the newest `pending_payment` order and exposes queue count plus next-step copy.

- [ ] **Step 2: Run the focused test to verify it fails**

Run: `node --test tests/portal-recharge-workflow-polish.test.mjs`

Expected: FAIL because the helper does not exist yet.

- [ ] **Step 3: Implement the minimal helper**

Add a helper that:

- reuses existing history-order sorting
- filters to pending-payment orders
- returns `null` when no pending-payment orders exist
- returns the newest pending order and queue metadata otherwise

- [ ] **Step 4: Re-run the focused test**

Run: `node --test tests/portal-recharge-workflow-polish.test.mjs`

Expected: PASS.

### Task 3: Render the Callout in the Recharge Page

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx`
- Reference: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/services/index.ts`

- [ ] **Step 1: Wire the pending-settlement view model into the page**

Consume the helper near the existing quote and pending-order derivations.

- [ ] **Step 2: Render a visible next-step callout near checkout summary**

Show:

- queue headline
- latest pending order amount
- latest pending order time
- direct billing CTA

- [ ] **Step 3: Keep the history section unchanged as the audit surface**

Do not move or weaken the table behavior. The new callout is guidance, not a replacement for history.

### Task 4: Verify the Full Portal Recharge Surface

**Files:**
- Verify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx`
- Verify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/services/index.ts`
- Verify: `apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs`
- Verify: `apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`
- Verify: `apps/sdkwork-router-portal/tests/portal-recharge-finance-projection.test.mjs`
- Verify: `apps/sdkwork-router-portal/tests/portal-runtime-dependency-governance.test.mjs`

- [ ] **Step 1: Run the portal recharge tests**

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

Expected: only intended recharge-page changes are present in the clean worktree.
