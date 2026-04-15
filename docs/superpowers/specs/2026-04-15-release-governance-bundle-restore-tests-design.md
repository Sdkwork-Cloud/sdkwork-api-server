# Release Governance Bundle Restore Tests Design

**Date:** 2026-04-15

## Goal

Ensure the pull-request `release-governance` workflow actually executes the direct regression tests for the governed artifact bundle and restore chain:

- `scripts/release/tests/materialize-release-governance-bundle.test.mjs`
- `scripts/release/tests/restore-release-governance-latest.test.mjs`

## Current Evidence

- `.github/workflows/release-governance.yml` watches `scripts/release/**`.
- The release workflow relies on `scripts/release/materialize-release-governance-bundle.mjs` to package a single governance bundle artifact for operators.
- The bundle manifest explicitly points operators to `scripts/release/restore-release-governance-latest.mjs`.
- Direct regression tests already exist for both helpers:
  - `scripts/release/tests/materialize-release-governance-bundle.test.mjs`
  - `scripts/release/tests/restore-release-governance-latest.test.mjs`
- `scripts/release/run-release-governance-checks.mjs` preflight currently executes the release workflow contract test, but it does not execute the direct bundle or restore regression files.

## Problem Statement

The PR governance workflow claims the full `scripts/release/**` surface as release-governed input, but the operator restore chain is only covered indirectly through higher-level workflow assertions.

That leaves a real review blind spot:

- a PR that changes the bundle helper or restore helper triggers `release-governance`
- the preflight gate does not run the direct regression tests for those helpers
- CI therefore proves that the workflow still references the chain, but not that the chain still behaves correctly

For commercial delivery, this is a release recoverability risk. If the bundle output or restore semantics regress, operators can lose the ability to replay governed evidence on blocked hosts even though PR governance appeared to pass.

## Options Considered

### Option A: Keep relying on `release-workflow.test.mjs`

Pros:

- no extra preflight runtime

Cons:

- preserves the watched-test versus executed-test mismatch
- only proves workflow wiring, not bundle/restore helper behavior

### Option B: Stop treating the bundle and restore helpers as release-governed PR inputs

Pros:

- removes the mismatch mechanically

Cons:

- weakens governance around operator recovery tooling
- conflicts with the existing `scripts/release/**` watch surface

### Option C: Add the direct bundle and restore regression tests to the release-governance preflight plan

Pros:

- aligns the watched release-helper surface with executed regression coverage
- hardens the operator recovery chain without broadening the workflow contract unnecessarily
- keeps the fix narrowly focused on governance execution gaps

Cons:

- slightly increases preflight runtime

## Recommendation

Choose Option C.

The recommended change is to add explicit preflight plan ids for the bundle and restore regression files, then extend the runner contract tests so this coverage cannot silently disappear again.

## Verification Boundary

This slice is acceptable only if:

- `node --test scripts/release/tests/release-governance-runner.test.mjs` passes
- `node --test scripts/release/tests/materialize-release-governance-bundle.test.mjs scripts/release/tests/restore-release-governance-latest.test.mjs scripts/release/tests/release-governance-runner.test.mjs` passes
- `node scripts/release/run-release-governance-checks.mjs --profile preflight --format json` reports `ok: true`

## Success Condition

This work is successful when a PR that changes the release-governance bundle or restore helpers both triggers `release-governance` and executes their direct regression coverage during preflight.
