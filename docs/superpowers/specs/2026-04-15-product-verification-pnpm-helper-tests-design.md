# Product Verification Pnpm Helper Tests Design

**Date:** 2026-04-15

## Goal

Ensure the pull-request `product-verification` workflow actually executes the watched `pnpm-launch-lib` helper tests.

## Current Evidence

- `.github/workflows/product-verification.yml` watches:
  - `scripts/dev/pnpm-launch-lib.mjs`
  - `scripts/dev/tests/pnpm-launch-lib.test.mjs`
- the workflow's "Run product governance node tests" step currently runs:
  - `scripts/product-verification-workflow.test.mjs`
  - `scripts/check-router-product.test.mjs`
  - `scripts/build-router-desktop-assets.test.mjs`
  - `scripts/check-router-docs-safety.test.mjs`
  - `scripts/check-router-frontend-budgets.test.mjs`
  - `apps/sdkwork-router-portal/tests/product-entrypoint-scripts.test.mjs`
- the workflow does not currently execute `scripts/dev/tests/pnpm-launch-lib.test.mjs`.

## Problem Statement

The product verification workflow is already treating the shared pnpm helper test as a governance input at trigger time, but not at execution time.

That creates a review-time blind spot:

- PRs that modify the pnpm helper contract test still trigger the workflow
- the workflow run does not actually execute that changed contract test
- shared frontend install/runtime logic can drift away from its direct regression coverage while CI still looks complete

This is a commercial-readiness problem because the workflow claims governance coverage over a helper contract that it does not run.

## Options Considered

### Option A: Stop watching `scripts/dev/tests/pnpm-launch-lib.test.mjs`

Pros:

- removes the trigger/execution mismatch

Cons:

- weakens governance over a shared frontend runtime helper
- throws away an existing regression asset instead of using it

### Option B: Rely only on indirect coverage through product-check tests

Pros:

- no workflow command growth

Cons:

- indirect tests do not cover all helper-specific behaviors
- leaves the watched-test mismatch unresolved

### Option C: Add `scripts/dev/tests/pnpm-launch-lib.test.mjs` to the workflow's Node test step

Pros:

- aligns trigger scope with executed verification
- reuses an existing direct helper regression suite
- keeps the fix narrowly scoped to CI governance

Cons:

- slightly longer Node test step

## Recommendation

Choose Option C.

The trustworthy fix is to make `product-verification` execute the same shared pnpm helper contract test that it already treats as a pull-request trigger input.

## Verification Boundary

This slice is acceptable only if:

- `node --test scripts/product-verification-workflow.test.mjs` passes
- `node --test scripts/product-verification-workflow.test.mjs scripts/check-router-product.test.mjs scripts/build-router-desktop-assets.test.mjs scripts/check-router-docs-safety.test.mjs scripts/check-router-frontend-budgets.test.mjs scripts/dev/tests/pnpm-launch-lib.test.mjs apps/sdkwork-router-portal/tests/product-entrypoint-scripts.test.mjs` passes

## Success Condition

This work is successful when the `product-verification` workflow fails if the pnpm helper contract test disappears from its Node test step and any PR that changes the watched helper test also executes that test during CI.
