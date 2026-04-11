# Portal Recharge Simplification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** simplify the portal recharge page into a commercial SaaS purchase surface with only recharge options, payment information, and recharge history.

**Architecture:** keep the existing recharge quote and order creation flow, but remove the summary-grid and decision-support page contract entirely. Simplify the recharge page data shape so it only fetches data required by the remaining three-section layout, then restyle the surviving purchase flow to feel more premium and conversion-oriented.

**Tech Stack:** React 19, TypeScript, portal package tests with Node test runner, portal shared UI primitives, portal repository helpers.

---

## File Map

- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs`
  - update the page contract to require only the three surviving sections and the new payment-information focus.
- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-finance-projection.test.mjs`
  - remove the obsolete finance-projection page assertions and replace them with simplified recharge page assertions.
- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`
  - keep workflow behavior assertions that still matter after the redesign.
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/repository/index.ts`
  - remove recharge-page-only data fetches that supported deleted decision-support UI.
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/types/index.ts`
  - simplify the recharge page data shape and remove page-only finance-projection types if no longer needed.
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/services/index.ts`
  - remove page-only summary/finance helpers if they are no longer used after the simplified layout.
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx`
  - rebuild the page into a three-section, premium commercial purchase surface.

### Task 1: Lock the simplified recharge contract in tests

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-finance-projection.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`

- [ ] **Step 1: Write the failing contract assertions**

Update the tests so they require:

- `data-slot="portal-recharge-options"`
- `data-slot="portal-recharge-quote-card"`
- `data-slot="portal-recharge-history-table"`
- `Payment information`
- `Recharge options`
- `Recharge history`

And reject:

- `data-slot="portal-recharge-summary-grid"`
- `data-slot="portal-recharge-decision-support"`
- `data-slot="portal-recharge-multimodal-demand"`
- `Recharge decision support`

- [ ] **Step 2: Run the recharge-focused tests to verify they fail**

Run:

```bash
node --test apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs apps/sdkwork-router-portal/tests/portal-recharge-finance-projection.test.mjs apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs
```

Expected: FAIL because the current page still renders the removed summary and decision-support sections.

- [ ] **Step 3: Commit the red-state test update if working in a dedicated branch**

```bash
git add apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs apps/sdkwork-router-portal/tests/portal-recharge-finance-projection.test.mjs apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs
git commit -m "test: lock simplified portal recharge contract"
```

### Task 2: Remove obsolete recharge-page-only data dependencies

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/repository/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/types/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/services/index.ts`

- [ ] **Step 1: Write the failing cleanup expectations in the updated tests**

The updated tests from Task 1 should also stop expecting:

- `getPortalCommerceMembership` in the recharge repository
- `getPortalBillingEventSummary` in the recharge repository
- recharge-page-only finance-projection types and page copy

- [ ] **Step 2: Run the focused tests again to verify the cleanup expectations fail**

Run:

```bash
node --test apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs apps/sdkwork-router-portal/tests/portal-recharge-finance-projection.test.mjs
```

Expected: FAIL because the repository and page types still include the deleted decision-support data path.

- [ ] **Step 3: Implement the minimal cleanup**

Apply these changes:

- remove `getPortalCommerceMembership()` and `getPortalBillingEventSummary()` from the recharge page repository loader
- remove `membership` and `billing_event_summary` from `PortalRechargePageData`
- remove `PortalRechargeSummaryCard`, `PortalRechargeMultimodalTotals`, and `PortalRechargeFinanceProjection` if they are no longer used
- remove `buildPortalRechargeFinanceProjection()` and `buildPortalRechargeSummaryCards()` if nothing else imports them after the page rewrite
- keep quote snapshot, history-row, validation, and amount parsing helpers

- [ ] **Step 4: Run the focused tests to verify the cleanup passes**

Run:

```bash
node --test apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs apps/sdkwork-router-portal/tests/portal-recharge-finance-projection.test.mjs
```

Expected: PASS for the structural cleanup expectations.

- [ ] **Step 5: Commit the data-path cleanup**

```bash
git add apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/repository/index.ts apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/types/index.ts apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/services/index.ts apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs apps/sdkwork-router-portal/tests/portal-recharge-finance-projection.test.mjs
git commit -m "refactor: simplify portal recharge page data flow"
```

### Task 3: Rebuild the recharge page into a premium three-section purchase flow

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`

- [ ] **Step 1: Write the failing page-level assertions for the new purchase presentation**

Add or update assertions so the page source requires:

- a premium-highlighted recommended recharge card
- `Payment information` as the quote-panel heading
- a single primary order-creation CTA
- recharge history as a lower-priority section

Keep assertions for:

- `Custom amount`
- `Create recharge order`
- `Pending payment queue`
- `Open billing workbench`

- [ ] **Step 2: Run the page-contract tests to verify they fail**

Run:

```bash
node --test apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs
```

Expected: FAIL because the current page still uses the old workspace-style layout and quote-panel copy.

- [ ] **Step 3: Implement the page rewrite**

Update `index.tsx` so it:

- removes summary-grid rendering
- removes decision-support rendering
- keeps one main purchase section for preset cards plus custom amount
- moves the quote card into a cleaner `Payment information` confirmation panel
- emphasizes the recommended option and selected state with stronger commercial styling
- preserves custom amount preview flow and recharge order creation
- preserves recharge history table and billing navigation as a secondary area

- [ ] **Step 4: Run the recharge-focused tests to verify the page passes**

Run:

```bash
node --test apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs apps/sdkwork-router-portal/tests/portal-recharge-finance-projection.test.mjs apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs
```

Expected: PASS.

- [ ] **Step 5: Run portal typecheck to verify the page compiles cleanly**

Run:

```bash
pnpm --dir apps/sdkwork-router-portal typecheck
```

Expected: PASS with no type errors from the recharge package changes.

- [ ] **Step 6: Commit the page redesign**

```bash
git add apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs apps/sdkwork-router-portal/tests/portal-recharge-finance-projection.test.mjs apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs
git commit -m "feat: simplify portal recharge purchase flow"
```

### Task 4: Final verification

**Files:**
- Modify: none

- [ ] **Step 1: Run the full recharge-targeted verification set**

Run:

```bash
node --test apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs apps/sdkwork-router-portal/tests/portal-recharge-finance-projection.test.mjs apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs
pnpm --dir apps/sdkwork-router-portal typecheck
```

Expected: PASS.

- [ ] **Step 2: Review the final diff**

Run:

```bash
git diff -- apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/repository/index.ts apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/types/index.ts apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/services/index.ts apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs apps/sdkwork-router-portal/tests/portal-recharge-finance-projection.test.mjs apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs
```

Expected: the diff only reflects the simplified three-section purchase flow and its supporting cleanup.
