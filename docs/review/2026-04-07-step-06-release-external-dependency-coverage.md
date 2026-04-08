# 2026-04-07 Step 06 Release External Dependency Coverage Review

## Scope

This review slice continued Wave `B` / Step `06` on the release-governance lane and tightened the release dependency materialization truth.

Primary target in this round:

- prove that every current release-app sibling dependency reference is covered by the declared GitHub materialization specs

Execution boundary:

- do not expand the release dependency model without evidence
- do not change local development dependency wiring
- do not commit, push, tag, or publish a GitHub release

## Decision Ledger

- Date: `2026-04-07`
- Wave / Step: `B / 06`
- Primary mode: `release-gate-hardening`
- Previous mode: `release-gate-hardening`
- Strategy switch: no

### Top 3 Candidate Actions

1. Add a coverage audit that scans release-app sibling references and verifies they are all backed by declared materialization specs.
   - `Priority Score: 89`
   - closes a real silent-drift risk in the current release path
2. Expand the materialization spec list to all governed sibling repositories without evidence that the current release graph uses them.
   - `Priority Score: 54`
   - rejected because it would widen the release model without factual need
3. Stop at the existing `sdkwork-ui` helper and leave future sibling drift to manual review.
   - `Priority Score: 37`
   - rejected because it preserves a hidden regression lane

### Chosen Action

Action 1 was selected because it turns the current release dependency scope into an executable fact instead of a manual assumption.

## Implemented Fixes

- updated `scripts/release/materialize-external-deps.mjs`
  - added recursive scanning for release-app dependency sources under:
    - `apps/sdkwork-router-admin`
    - `apps/sdkwork-router-portal`
  - audits external sibling references coming from:
    - `package.json` `file:` dependencies
    - `pnpm-workspace.yaml` package roots
    - `tsconfig.json` path mappings
  - maps each external reference back to a declared materialization spec
  - blocks the CLI if any external sibling reference is uncovered
- updated `scripts/release/release-workflow-contracts.mjs`
  - now asserts that external release dependency coverage is complete
  - now asserts the current covered external dependency ids are `['sdkwork-ui']`
- updated `scripts/release/tests/release-workflow.test.mjs`
  - verifies coverage exports
  - verifies the coverage result is green
  - verifies admin package, portal workspace, and portal tsconfig all contribute audited external references
- updated `docs/release/README.md`
  - documented the coverage-audit rule and current bounded dependency surface

## Verification Evidence

- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
- `node scripts/release/materialize-external-deps.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`

Observed result:

- the coverage audit is green
- the current release-app external sibling dependency surface resolves only to `sdkwork-ui`
- the helper remains non-destructive in the current workspace and reports:
  - `reused sdkwork-ui from Sdkwork-Cloud/sdkwork-ui@main`

## Current Assessment

### Closed In This Slice

- the release path no longer relies on a silent assumption that `sdkwork-ui` is the only external sibling dependency
- future introduction of unmanaged sibling repository references into admin / portal release apps will now fail contract verification

### Still Open

- live repository sync truth is still blocked
- no commit / push / GitHub release is authorized
- broader governed repositories remain part of the release-truth gate even if they are not currently in the release-app dependency graph

## Next Slice Recommendation

1. Keep the coverage audit as the boundary for release-app sibling materialization.
2. Continue using `run-release-governance-checks.mjs` as the primary release-gate entry point.
3. Do not widen the materialization spec list unless the audited release graph proves a new external sibling dependency exists.
