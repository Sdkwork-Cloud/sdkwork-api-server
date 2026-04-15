# Release Governance Telemetry Evidence Tests Design

**Date:** 2026-04-15

## Goal

Ensure the pull-request `release-governance` workflow actually executes the direct regression tests for the governed telemetry evidence generation chain:

- `scripts/release/tests/materialize-release-telemetry-export.test.mjs`
- `scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs`
- `scripts/release/tests/materialize-slo-governance-evidence.test.mjs`

## Current Evidence

- `.github/workflows/release-governance.yml` watches `scripts/release/**`.
- The release workflow depends on three repository-owned helpers to materialize governed evidence:
  - `materialize-release-telemetry-export.mjs`
  - `materialize-release-telemetry-snapshot.mjs`
  - `materialize-slo-governance-evidence.mjs`
- Direct regression tests already exist for all three helpers.
- `scripts/release/run-release-governance-checks.mjs` preflight currently exercises the higher-level SLO governance checks, but it does not execute the direct telemetry export, telemetry snapshot, or SLO evidence materializer regressions.

## Problem Statement

The PR governance workflow claims the full `scripts/release/**` surface as governed release input, but the evidence-generation chain is still covered mainly through indirect workflow and SLO checks.

That leaves a concrete review gap:

- a PR that changes one of the direct evidence materializers triggers `release-governance`
- the preflight gate does not execute that helper's direct regression test
- CI proves only that higher-level lanes still exist, not that the governed evidence chain still behaves correctly at the helper boundary

For commercial delivery, this is a release-auditability and recoverability risk. If telemetry export, derived snapshot, or governed SLO evidence generation regresses, operators can receive incomplete or malformed evidence while PR governance still appears green.

## Options Considered

### Option A: Keep relying on `release-slo-governance.test.mjs` and `release-workflow.test.mjs`

Pros:

- no additional preflight runtime

Cons:

- preserves the watched-test versus executed-test mismatch
- weakens fault localization when evidence generation regresses

### Option B: Add only one direct telemetry helper test at a time

Pros:

- smallest incremental runtime increase

Cons:

- leaves the rest of the evidence chain indirectly covered
- keeps the chain only partially governed at PR time

### Option C: Add the direct telemetry export, snapshot, and SLO evidence regression tests together

Pros:

- aligns the watched evidence-generation surface with executed regression coverage
- treats the governed evidence chain as one cohesive release boundary
- keeps the change limited to runner coverage rather than expanding workflow triggers

Cons:

- slightly increases preflight runtime

## Recommendation

Choose Option C.

The telemetry export, telemetry snapshot, and governed SLO evidence helpers form one release-evidence pipeline. The trustworthy fix is to add all three direct regression files to the release-governance preflight plan and strengthen runner contract tests so this chain cannot silently drop out of PR coverage again.

## Verification Boundary

This slice is acceptable only if:

- `node --test scripts/release/tests/release-governance-runner.test.mjs` passes
- `node --test scripts/release/tests/materialize-release-telemetry-export.test.mjs scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs scripts/release/tests/materialize-slo-governance-evidence.test.mjs scripts/release/tests/release-governance-runner.test.mjs` passes
- `node scripts/release/run-release-governance-checks.mjs --profile preflight --format json` reports `ok: true`

## Success Condition

This work is successful when a PR that changes any release telemetry evidence materializer both triggers `release-governance` and executes that helper's direct regression during preflight.
