# Product Verification Workflow Contract Paths Design

**Date:** 2026-04-15

## Goal

Ensure the pull-request `product-verification` workflow remains self-protecting by treating `scripts/product-verification-workflow-contracts.mjs` as a required watched path in workflow contract tests.

## Current Evidence

- `.github/workflows/product-verification.yml` already watches `scripts/product-verification-workflow-contracts.mjs`.
- `scripts/product-verification-workflow-contracts.mjs` does not currently assert that watched path.
- `scripts/product-verification-workflow.test.mjs` also does not currently require it.

## Problem Statement

The workflow YAML is correct today, but the contract layer does not enforce that correctness. A future edit can remove `scripts/product-verification-workflow-contracts.mjs` from `pull_request.paths` and still pass the current workflow tests.

For commercial CI governance, this is a latent regression channel:

- product-governance changes can stop triggering PR verification
- the workflow appears protected, but the protection is not asserted
- governance intent depends on manual vigilance instead of executable contracts

## Options Considered

### Option A: Leave the workflow as-is and rely on reviewers

Pros:

- no code change

Cons:

- not mechanically enforced
- regression can reappear silently

### Option B: Enforce the contract helper path in the workflow test and contract helper

Pros:

- converts intent into executable policy
- keeps the fix localized to workflow governance files
- mirrors the hardening already applied to `release-governance`

Cons:

- adds one more assertion and one rejecting fixture

## Recommendation

Choose Option B.

The YAML already contains the right path. The missing piece is executable enforcement. The clean fix is to require that path in both the direct workflow test and the contract helper.

## Design

### Contract Boundary

Strengthen `scripts/product-verification-workflow-contracts.mjs` so it asserts the workflow watches:

- `.github/workflows/product-verification.yml`
- `scripts/product-verification-workflow-contracts.mjs`
- `scripts/product-verification-workflow.test.mjs`

### Test Boundary

Strengthen `scripts/product-verification-workflow.test.mjs` with:

- a direct assertion that the real workflow contains `scripts/product-verification-workflow-contracts.mjs`
- a rejecting fixture proving the helper fails when that watched path is removed

### Verification Boundary

The slice is acceptable only if:

- `node --test scripts/product-verification-workflow.test.mjs` passes
- `node --test scripts/check-router-product.test.mjs scripts/build-router-desktop-assets.test.mjs scripts/check-router-docs-safety.test.mjs scripts/check-router-frontend-budgets.test.mjs apps/sdkwork-router-portal/tests/product-entrypoint-scripts.test.mjs` still passes

## Success Condition

This work is successful when removing the contract-helper watched path from `product-verification.yml` causes workflow contract tests to fail immediately.
