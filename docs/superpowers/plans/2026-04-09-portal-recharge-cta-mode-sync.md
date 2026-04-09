# Portal Recharge CTA Mode Sync Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** make the recharge page use a single coherent CTA hierarchy across purchase mode and post-order handoff mode.

**Architecture:** Keep the existing post-order handoff state, but route both desktop and mobile CTA labels plus click behavior through that same state check. Add one explicit secondary exit action to clear handoff mode and return to purchase mode intentionally.

**Tech Stack:** React 19, TypeScript, Node test runner, existing portal commons UI

---

### Task 1: Lock the CTA Mode Contract

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`

- [ ] **Step 1: Add failing assertions**

Add assertions for:

- `Create another order`
- desktop primary CTA using `postOrderHandoffActive`
- `Continue in billing` remaining the handoff action

- [ ] **Step 2: Run focused tests to verify they fail**

Run: `node --test tests/portal-recharge-center.test.mjs tests/portal-recharge-workflow-polish.test.mjs`

Expected: FAIL because the desktop CTA and secondary exit path are not fully implemented yet.

### Task 2: Implement CTA Mode Switching

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx`

- [ ] **Step 1: Route the desktop primary CTA through handoff state**

When `postOrderHandoffActive` is true, the primary CTA label and action should switch to billing continuation.

- [ ] **Step 2: Add a secondary `Create another order` action**

Render it in the handoff panel and clear the local handoff state when clicked.

- [ ] **Step 3: Keep purchase mode unchanged**

When handoff is inactive, the page must still create a recharge order with the current selection.

### Task 3: Verify the Full Portal Recharge Surface

**Files:**
- Verify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx`
- Verify: `apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs`
- Verify: `apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`
- Verify: `apps/sdkwork-router-portal/tests/portal-recharge-finance-projection.test.mjs`
- Verify: `apps/sdkwork-router-portal/tests/portal-runtime-dependency-governance.test.mjs`

- [ ] **Step 1: Run portal recharge tests**

Run: `node --test tests/portal-recharge-center.test.mjs tests/portal-recharge-workflow-polish.test.mjs tests/portal-recharge-finance-projection.test.mjs tests/portal-runtime-dependency-governance.test.mjs`

Expected: PASS.

- [ ] **Step 2: Run typecheck**

Run: `pnpm typecheck`

Expected: PASS.

- [ ] **Step 3: Run production build**

Run: `pnpm build`

Expected: PASS.
