# Release Governance Materialize External Deps Test Design

**Date:** 2026-04-15

## Goal

Ensure the pull-request `release-governance` workflow actually executes the direct regression test for `scripts/release/materialize-external-deps.mjs`.

## Current Evidence

- `.github/workflows/release-governance.yml` watches `scripts/release/**`.
- `scripts/release/tests/materialize-external-deps.test.mjs` now exists under that watched path surface.
- `scripts/release/run-release-governance-checks.mjs` preflight currently executes:
  - `release-sync-audit-test`
  - `release-workflow-test`
  - `release-governance-workflow-test`
  - `release-attestation-verify-test`
  - `release-observability-test`
  - `release-slo-governance-contracts-test`
  - `release-slo-governance-test`
  - `release-runtime-tooling-test`
  - `release-window-snapshot-test`
- the preflight plan does not execute `scripts/release/tests/materialize-external-deps.test.mjs`.

## Problem Statement

The `release-governance` PR workflow already treats the entire `scripts/release/**` tree as a governance trigger surface, but one of the newly added direct helper regressions is not actually run by the preflight gate.

That creates the same class of drift we already closed elsewhere:

- a PR that changes `materialize-external-deps.mjs` or its direct test necessarily triggers `release-governance`
- the workflow run does not execute the direct helper regression for that behavior
- CI relies only on indirect coverage through larger release tests, which weakens review visibility for a governed release helper

This is a commercial-governance gap because the workflow claims PR-time governance over the helper surface without executing the direct regression that documents it.

## Options Considered

### Option A: Keep relying on indirect coverage through `release-workflow.test.mjs`

Pros:

- no additional preflight step

Cons:

- leaves the watched-test/executed-test mismatch intact
- makes helper regressions harder to localize

### Option B: Stop watching the direct helper regression file

Pros:

- removes the mismatch mechanically

Cons:

- weakens CI governance over an important release helper
- throws away useful direct regression coverage

### Option C: Add `materialize-external-deps.test.mjs` to the release-governance preflight plan

Pros:

- aligns watched PR inputs with executed regression coverage
- keeps the fix narrowly scoped to CI governance
- preserves both direct helper tests and broader release workflow tests

Cons:

- slightly increases preflight runtime

## Recommendation

Choose Option C.

The smallest trustworthy fix is to add a dedicated preflight test plan for `scripts/release/tests/materialize-external-deps.test.mjs` and lock it into the workflow/runner contract tests.

## Verification Boundary

This slice is acceptable only if:

- `node --test scripts/release-governance-workflow.test.mjs scripts/release/tests/release-governance-runner.test.mjs` passes
- `node --test scripts/release/tests/materialize-external-deps.test.mjs scripts/release-governance-workflow.test.mjs scripts/release/tests/release-governance-runner.test.mjs` passes
- `node scripts/release/run-release-governance-checks.mjs --profile preflight --format json` reports `ok: true`

## Success Condition

This work is successful when a PR that changes the external release dependency materializer or its direct regression test both triggers `release-governance` and causes that direct regression to be executed during preflight.
