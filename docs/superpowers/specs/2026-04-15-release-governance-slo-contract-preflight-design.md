# Release Governance SLO Contract Preflight Design

**Date:** 2026-04-15

## Goal

Promote SLO governance contract validation from a fallback-only safeguard into the normal `release-governance` preflight path.

## Current Evidence

- `scripts/release/run-release-governance-checks.mjs` runs `release-slo-governance.test.mjs` during preflight.
- the runner only invokes `assertSloGovernanceContracts` when child execution is blocked and it falls back in-process.
- `assertSloGovernanceContracts` requires two governed architecture baselines:
  - `docs/架构/135-可观测性与SLO治理设计-2026-04-07.md`
  - `docs/架构/143-全局架构对齐与收口计划-2026-04-08.md`
- `.github/workflows/release-governance.yml` does not currently watch those two documents.

## Problem Statement

This creates an asymmetric governance model:

- normal CI preflight validates SLO behavior, but not the declared contract baselines
- fallback mode validates more than normal mode
- edits to the governed architecture baselines can bypass PR-time release governance

For commercial delivery this is backwards. Governance contracts must be strongest on the normal path, not only on degraded execution paths.

## Recommendation

Add a dedicated `release-slo-governance-contracts` test to the normal `release-governance` preflight sequence, and make the PR workflow watch the two governed architecture baseline documents.

This is the smallest fix that makes the normal path at least as strict as the fallback path.

## Design

### Runner Boundary

Add a new preflight plan entry:

- `release-slo-governance-contracts-test`

that runs:

- `node --test --experimental-test-isolation=none scripts/release/tests/release-slo-governance-contracts.test.mjs`

Its fallback implementation should call `assertSloGovernanceContracts`.

### Workflow Boundary

Update `.github/workflows/release-governance.yml` so `pull_request.paths` includes:

- `docs/架构/135-可观测性与SLO治理设计-2026-04-07.md`
- `docs/架构/143-全局架构对齐与收口计划-2026-04-08.md`

### Contract Boundary

Strengthen the workflow contract helper and workflow tests so they fail if those two document paths are no longer watched.

## Verification Boundary

The slice is acceptable only if:

- `node --test scripts/release-governance-workflow.test.mjs` passes
- `node --test scripts/release/tests/release-governance-runner.test.mjs scripts/release/tests/release-slo-governance-contracts.test.mjs` passes
- `node scripts/release/run-release-governance-checks.mjs --profile preflight --format json` returns `"ok": true`

## Success Condition

This work is successful when normal PR-time `release-governance` preflight explicitly validates the governed SLO architecture baselines and re-runs whenever those baseline documents change.
