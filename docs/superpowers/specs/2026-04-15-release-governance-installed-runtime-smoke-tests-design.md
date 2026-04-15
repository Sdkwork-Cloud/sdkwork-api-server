# Release Governance Installed Runtime Smoke Tests Design

**Date:** 2026-04-15

## Goal

Ensure the pull-request `release-governance` workflow executes the direct regression tests for the installed-runtime smoke helpers:

- `scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs`
- `scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs`

## Current Evidence

- The native release workflow already invokes:
  - `scripts/release/run-unix-installed-runtime-smoke.mjs`
  - `scripts/release/run-windows-installed-runtime-smoke.mjs`
- `scripts/release/tests/release-workflow.test.mjs` verifies those workflow steps and evidence upload wiring.
- Direct CLI-contract tests already exist for both smoke helpers.
- `scripts/release/run-release-governance-checks.mjs` preflight now covers the direct Git-derived materializer chain, telemetry evidence chain, governance bundle, restore path, and release workflow contracts.
- The remaining release direct tests not exercised by preflight are now:
  - `scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs`
  - `scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs`
  - `scripts/release/tests/release-governance-runner.test.mjs`

## Problem Statement

The release workflow advertises installed-runtime smoke evidence as a governed release signal, but PR preflight still does not execute the direct contract tests for the Unix and Windows smoke helpers themselves.

That leaves a narrow but real gap:

- a PR can change one of the installed-runtime smoke scripts
- `release-governance` preflight will trigger
- the direct helper regression will not run
- CI will only prove the workflow still references the helper, not that the helper still exposes the required CLI contract and evidence-shape behavior

For commercial delivery, this is a release-operability risk. A broken smoke helper can silently invalidate install-time release verification or evidence generation while the governance lane still appears healthy.

## Options Considered

### Option A: Keep relying on `release-workflow.test.mjs` and runtime-tooling coverage

Pros:

- no preflight runtime increase

Cons:

- preserves the watched-versus-executed mismatch for the smoke helper boundary
- weakens fault localization if install-time evidence generation regresses

### Option B: Add only one platform's direct smoke test

Pros:

- smallest incremental runtime increase

Cons:

- leaves the other platform indirectly covered
- guarantees another follow-up slice for the same boundary type

### Option C: Add both platform smoke tests together and use in-process fallback verification

Pros:

- closes the last meaningful release-helper execution gap outside runner self-tests
- keeps the fix local to the governance runner
- avoids attempting real installed-runtime launches on blocked hosts

Cons:

- slightly increases preflight runtime

## Recommendation

Choose Option C.

The Unix and Windows installed-runtime smoke helpers form one governed install-verification boundary. They should be added together so both platform contracts are explicitly exercised during preflight, and blocked-host fallback should validate their parse/plan/evidence contract in-process rather than trying to execute real runtime launches.

`release-governance-runner.test.mjs` should remain outside the release-governance plan. Pulling the runner's own test into the runner would create self-recursive governance and reduce clarity instead of increasing assurance.

## Verification Boundary

This slice is acceptable only if:

- `node --test scripts/release/tests/release-governance-runner.test.mjs` passes
- `node --test --experimental-test-isolation=none scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs` passes
- `node --test --experimental-test-isolation=none scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs` passes
- `node scripts/release/run-release-governance-checks.mjs --profile preflight --format json` reports `ok: true`

## Success Condition

This work is successful when a PR that changes either installed-runtime smoke helper both triggers `release-governance` and executes that helper's direct regression during preflight.
