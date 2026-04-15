# Release Governance Workflow Contract Paths Design

**Date:** 2026-04-15

## Goal

Ensure the pull-request `release-governance` workflow is re-run whenever its own contract helper changes, so governance policy cannot drift without CI re-validating the workflow surface.

## Current Evidence

- `.github/workflows/release-governance.yml` currently triggers on the workflow YAML, `scripts/release/**`, `scripts/release-governance-workflow.test.mjs`, `bin/**`, and `docs/release/**`.
- `scripts/release-governance-workflow-contracts.mjs` lives outside `scripts/release/**`.
- `scripts/release/run-release-governance-checks.mjs` already depends on `assertReleaseGovernanceWorkflowContracts`.
- `scripts/release-governance-workflow.test.mjs` currently verifies only the happy path and does not prove the workflow watches its contract helper.

## Problem Statement

The PR workflow exists, but its trigger surface is incomplete. A change that weakens `scripts/release-governance-workflow-contracts.mjs` can merge without running the `release-governance` workflow, because that file is outside the current path filter.

For commercial delivery, this is a governance bypass:

- workflow policy can drift without PR-time feedback
- the release-governance lane can appear healthy while its enforcement contract has already been weakened
- release-only validation becomes the first place the drift is rediscovered

## Options Considered

### Option A: Keep watching only the workflow test file

Pros:

- no workflow YAML change
- minimal maintenance

Cons:

- contract helper edits still bypass the PR workflow
- fallback contract validation in `run-release-governance-checks.mjs` is no longer protected at change time

### Option B: Add the contract helper to workflow paths and enforce it in workflow contract tests

Pros:

- closes the bypass directly
- keeps workflow triggers aligned with the actual enforcement surface
- makes the contract helper and workflow YAML mutually protected

Cons:

- adds one more watched file path
- requires a slightly stronger workflow test

### Option C: Move the contract helper under `scripts/release/**`

Pros:

- path filter would catch it implicitly

Cons:

- unnecessary file churn for a small governance fix
- does not improve test clarity by itself

## Recommendation

Choose Option B.

This is the smallest change that closes the CI bypass while preserving current structure. The workflow should explicitly watch `scripts/release-governance-workflow-contracts.mjs`, and the contract test should fail if that path disappears.

## Design

### Workflow Boundary

Update `.github/workflows/release-governance.yml` so the `pull_request.paths` list includes:

- `scripts/release-governance-workflow-contracts.mjs`

The workflow remains thin and should continue to delegate behavior to:

- `node scripts/release/run-release-governance-checks.mjs --profile preflight --format json`

### Contract Boundary

Strengthen `scripts/release-governance-workflow-contracts.mjs` so it asserts:

- the workflow exists
- it is triggered by `pull_request` and `workflow_dispatch`
- it watches `.github/workflows/release-governance.yml`
- it watches `scripts/release/**`
- it watches `scripts/release-governance-workflow-contracts.mjs`
- it watches `scripts/release-governance-workflow.test.mjs`
- it runs the preflight governance command

Strengthen `scripts/release-governance-workflow.test.mjs` with:

- direct assertions against the real workflow
- a rejecting fixture proving the helper fails when the contract module path is missing

### Verification Boundary

The slice is acceptable only if all of the following pass:

- `node --test scripts/release-governance-workflow.test.mjs`
- `node --test scripts/release/tests/release-governance-runner.test.mjs`
- `node scripts/release/run-release-governance-checks.mjs --profile preflight --format json`

## Risks And Mitigations

### Risk: path filters still miss adjacent governance files

Mitigation:

- explicitly watch the contract helper, not only the workflow test
- keep the workflow contract helper responsible for asserting watched governance files

### Risk: tests verify only happy-path presence

Mitigation:

- add a rejecting fixture so the helper proves it fails on missing contract coverage

## Success Condition

This work is successful when a PR that changes the release-governance workflow contract helper necessarily re-runs the `release-governance` workflow, and the workflow contract tests fail if that protection is removed.
