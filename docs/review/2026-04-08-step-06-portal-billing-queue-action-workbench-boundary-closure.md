# 2026-04-08 Step 06 Portal Billing Queue Action Workbench Boundary Closure Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by removing queue-row settlement ownership from the default pending-payment table while keeping the remaining explicit bridge/manual actions inside the already-opened checkout workbench.

Execution boundary:

- keep the existing Portal `Pending payment queue` and `Checkout session` workbench structure intact
- keep formal order / payment-method / payment-attempt truth primary
- do not invent a new backend operator console or new payment contracts
- do not remove the remaining explicit bridge/manual actions entirely; only move them behind the checkout workbench boundary

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `queue-action-workbench-boundary-closure`
- Previous mode: `operator-bridge-visibility-closure`
- Strategy switch: no

### Candidate Actions

1. Remove queue-row `settle` / `cancel` ownership and keep those actions only inside the opened checkout workbench.
   - `Priority Score: 181`
   - highest commercialization gain with the smallest UI and contract surface change

2. Delete the remaining `settle_order` / `cancel_order` actions from Portal billing entirely.
   - `Priority Score: 96`
   - rejected because the runtime still intentionally exposes those explicit bridge/manual actions and this slice did not change backend posture

3. Leave queue-row actions in place and rely on copy changes alone.
   - `Priority Score: 41`
   - rejected because the table would still act like the default settlement console even if the copy became softer

### Chosen Action

Action 1 was selected because the Portal runtime already has the correct explicit payment-action boundary inside the opened checkout workbench. Moving `settle` / `cancel` there reduces default queue-level bridge posture without widening scope or overstating backend readiness.

## Root Cause Summary

### 1. The Pending-Payment Queue Still Behaved Like A Settlement Console

Even after the earlier Step 06 slices established:

- canonical order detail
- canonical checkout-method composition
- canonical payment-attempt launch and history
- formal-first checkout presentation

the queue table still exposed direct `Settle order` and `Cancel order` row actions.

That kept the pending-payment inventory surface too close to a bridge/manual settlement console instead of a formal customer-facing payment queue.

### 2. The Checkout Workbench Already Had The Correct Context For Explicit Actions

The opened checkout workbench already carries:

- the selected order
- current payment rail posture
- checkout references
- payment-attempt history
- remaining method cards

That makes it the correct boundary for any remaining explicit `settle_order` or `cancel_order` action semantics.

### 3. Post-Order Copy Still Pointed Users Back Toward Queue-Level Settlement

After order creation, the status line still implied the queue itself was the next action surface. Once queue-row actions were removed, that guidance also needed to point users to the checkout workbench explicitly.

## Implemented Fixes

- updated the Portal billing page to:
  - introduce `activeCheckoutOrder` derived from `checkoutDetail?.order ?? null`
  - remove queue-row `handleQueueAction(row, 'settle')`
  - remove queue-row `handleQueueAction(row, 'cancel')`
  - keep `settle_order` and `cancel_order` method-card buttons in the checkout workbench, now bound to `handleQueueAction(activeCheckoutOrder, ...)`
  - change post-order status guidance to:
    - `{targetName} was queued in Pending payment queue. Open the checkout workbench to complete payment before quota or membership changes are applied.`
- updated shared Portal i18n registries and `zh-CN` translations for the new queue guidance copy
- updated source-contract tests to lock in:
  - `activeCheckoutOrder`
  - workbench-hosted settle / cancel actions
  - removal of queue-row settle / cancel actions
  - the new queue guidance copy
- repaired the i18n source-contract regex so it matches the actual multiline `t(...)` call shape used by the billing page

## Files Touched In This Slice

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts`
- `apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `docs/release/CHANGELOG.md`

## Verification Evidence

### Red First

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - failed before the test fix because the assertion only matched a single-line `t('...')` call while the page now uses a multiline `t(...)` invocation for the new queue guidance copy

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## Current Assessment

### Closed In This Slice

- the default `Pending payment queue` no longer owns `settle` / `cancel` as row-level actions
- the opened checkout workbench now owns the remaining explicit `settle_order` / `cancel_order` actions
- post-order billing guidance now points users to the checkout workbench instead of implying queue-level settlement
- shared Portal i18n and source-contract coverage now match the new workbench-boundary behavior

### Still Open

- explicit bridge/manual actions still remain inside the checkout workbench
- payment simulation posture still exists as an intentional bridge lane
- compatibility checkout-session is still retained as a fallback source
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/é‹èˆµç€¯/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal interaction boundaries, page copy, and source-contract coverage; it did not change route authority, runtime ownership, backend API truth, or architecture contracts

## Next Slice Recommendation

1. Continue reducing compatibility bridge ownership in the default billing user journey without deleting runtime-supported operator actions prematurely.
2. Evaluate whether the remaining workbench-only explicit actions should move into a clearer operator-only lane once formal provider coverage is sufficient.
3. Continue Step 06 commercialization closure until release evidence shows that pending-payment handling is formal-first end to end.
