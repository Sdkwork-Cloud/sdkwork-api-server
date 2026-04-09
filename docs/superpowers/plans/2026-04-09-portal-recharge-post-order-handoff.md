# Portal Recharge Post-Order Handoff Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** make the recharge page switch into a clearer billing handoff state immediately after a successful order creation.

**Architecture:** Keep order creation behavior unchanged, but track the newest created order id in the page session and use it to render a dedicated post-order handoff panel plus an adaptive mobile CTA. The handoff state should clear when the user begins a new purchase decision.

**Tech Stack:** React 19, TypeScript, Node test runner, existing portal commons UI

---

### Task 1: Lock the Post-Order Handoff Contract

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`

- [ ] **Step 1: Add failing UI-contract assertions**

Add assertions for:

- `data-slot="portal-recharge-post-order-handoff"`
- `Order ready for payment`
- `Continue in billing`

- [ ] **Step 2: Run focused tests to verify they fail**

Run: `node --test tests/portal-recharge-center.test.mjs tests/portal-recharge-workflow-polish.test.mjs`

Expected: FAIL because the new handoff panel does not exist yet.

### Task 2: Implement Post-Order Session State

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx`

- [ ] **Step 1: Track the newest created order id**

Capture the order returned from `createPortalRechargeOrder` and save its id in local state.

- [ ] **Step 2: Clear handoff state when the user starts a fresh selection**

Reset the local handoff state when the user changes preset choice or previews a new custom amount.

- [ ] **Step 3: Render the post-order handoff panel**

Show the panel only when:

- a local created-order id exists
- the latest pending-payment order matches that id

- [ ] **Step 4: Adapt the mobile sticky CTA**

When the post-order handoff is active, the mobile sticky CTA should prioritize `Continue in billing`.

### Task 3: Verify the Full Portal Recharge Surface

**Files:**
- Verify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx`
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

Expected: only intended recharge-page and spec/plan changes are present.
