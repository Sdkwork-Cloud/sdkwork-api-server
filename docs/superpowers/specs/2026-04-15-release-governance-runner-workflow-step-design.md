# Release Governance Runner Workflow Step Design

**Date:** 2026-04-15

## Goal

Ensure the `release-governance` GitHub workflow directly executes `scripts/release/tests/release-governance-runner.test.mjs` as an explicit workflow step, without recursively adding that test to the runner's own `preflight` plan.

## Current Evidence

- `scripts/release/run-release-governance-checks.mjs --profile preflight` now executes every release direct test under `scripts/release/tests/*.test.mjs` except one.
- The only remaining release direct test not directly executed by the workflow is:
  - `scripts/release/tests/release-governance-runner.test.mjs`
- `release-governance-runner.test.mjs` is intentionally not part of the runner's own plan because pulling it into `preflight` would create self-recursive governance.
- `.github/workflows/release-governance.yml` currently runs only:
  - `node scripts/release/run-release-governance-checks.mjs --profile preflight --format json`

## Problem Statement

The release-governance workflow now exercises the entire direct release-helper surface except the runner's own contract suite.

That leaves a narrow but meaningful gap:

- a PR can change `run-release-governance-checks.mjs` or `release-governance-runner.test.mjs`
- the workflow triggers because `scripts/release/**` is watched
- the workflow executes preflight, but not the runner's explicit contract suite
- CI therefore proves the current runner behavior still passes its own selected checks, but not that the runner contract around ordering, summaries, and fallback semantics remains intact

For commercial delivery, that is a governance-maintainability risk. The runner is the control point that wires the entire release-governance plan together. Its explicit self-test should be executed by CI, but it must be executed externally rather than recursively.

## Options Considered

### Option A: Add `release-governance-runner.test.mjs` to `preflight`

Pros:

- keeps all checks behind one runner entrypoint

Cons:

- creates self-recursive governance
- muddies failure ownership
- weakens the conceptual boundary between runner implementation and workflow orchestration

### Option B: Keep the runner test out of workflow execution

Pros:

- no workflow runtime increase

Cons:

- leaves the last release direct test unexecuted by CI
- weakens change detection for the governance control plane itself

### Option C: Add a dedicated workflow step that runs `release-governance-runner.test.mjs` before preflight

Pros:

- closes the final direct-test execution gap
- avoids self-recursion
- keeps failure ownership clear: runner self-test first, full preflight second

Cons:

- slightly increases workflow runtime

## Recommendation

Choose Option C.

The workflow should run the runner's explicit contract suite as a separate Node test step, then run `preflight`. This preserves architectural clarity and closes the final “watched but not directly workflow-executed” release test gap.

## Verification Boundary

This slice is acceptable only if:

- `node --test scripts/release-governance-workflow.test.mjs` passes
- `node --test scripts/release/tests/release-governance-runner.test.mjs` passes
- `.github/workflows/release-governance.yml` contains a dedicated step that runs `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`

## Success Condition

This work is successful when a PR that changes the release-governance runner or its contract suite triggers CI and directly executes the runner self-test before the `preflight` gate.
