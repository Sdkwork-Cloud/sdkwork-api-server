# Release Governance Window And Sync Materializer Tests Design

**Date:** 2026-04-15

## Goal

Ensure the pull-request `release-governance` workflow actually executes the direct regression tests for the governed Git-derived artifact materializers:

- `scripts/release/tests/materialize-release-window-snapshot.test.mjs`
- `scripts/release/tests/materialize-release-sync-audit.test.mjs`

## Current Evidence

- `.github/workflows/release-governance.yml` already watches `scripts/release/**`.
- The release evidence chain already materializes two Git-derived governed artifacts:
  - `release-window-snapshot-latest.json`
  - `release-sync-audit-latest.json`
- Direct regression tests already exist for both materializers.
- `scripts/release/run-release-governance-checks.mjs` preflight currently executes:
  - `release-window-snapshot.test.mjs`
  - `release-sync-audit.test.mjs`
  - telemetry evidence materializer tests
  - governance bundle and restore tests
- Preflight still does not execute the two direct materializer regressions that wrap the governed snapshot and sync-audit artifacts.

## Problem Statement

The PR governance workflow claims the full release evidence surface as governed input, but two core producer boundaries remain covered only indirectly.

That leaves a concrete gap:

- a PR can change `materialize-release-window-snapshot.mjs` or `materialize-release-sync-audit.mjs`
- `release-governance` preflight will trigger
- the direct materializer regressions will not run
- CI will prove surrounding helpers and downstream bundle logic still exist, but not that the artifact-producing boundary still behaves correctly

For commercial delivery, that is a release recoverability and auditability risk. If either materializer regresses, operators can lose a governed latest artifact or ship malformed evidence while the PR governance lane still reports green.

## Options Considered

### Option A: Keep relying on compute, verify, workflow, and bundle coverage

Pros:

- no additional preflight runtime

Cons:

- preserves the watched-versus-executed mismatch at the materializer boundary
- weakens defect localization when the artifact wrapper regresses

### Option B: Add only one of the two materializer regressions

Pros:

- smaller runtime increase than closing both gaps

Cons:

- leaves the Git-derived evidence chain only partially governed
- guarantees another near-identical follow-up slice

### Option C: Add both materializer regressions together and provide blocked-host fallback coverage

Pros:

- closes the remaining Git-derived artifact producer gap in one slice
- keeps the fix local to the runner rather than widening workflow scope
- preserves deterministic verification on hosts where spawning child Node processes is denied

Cons:

- slightly increases preflight runtime

## Recommendation

Choose Option C.

`release-window-snapshot` and `release-sync-audit` are the last Git-derived governed artifact producers still missing direct preflight execution. They should land together, and the runner contract must explicitly defend both the normal plan sequence and the blocked-host fallback behavior so the gap cannot silently reopen.

## Verification Boundary

This slice is acceptable only if:

- `node --test scripts/release/tests/release-governance-runner.test.mjs` passes
- `node --test scripts/release/tests/materialize-release-window-snapshot.test.mjs scripts/release/tests/materialize-release-sync-audit.test.mjs scripts/release/tests/release-governance-runner.test.mjs` passes
- `node scripts/release/run-release-governance-checks.mjs --profile preflight --format json` reports `ok: true`

## Success Condition

This work is successful when a PR that changes either Git-derived release artifact materializer both triggers `release-governance` and executes that helper's direct regression during preflight.
