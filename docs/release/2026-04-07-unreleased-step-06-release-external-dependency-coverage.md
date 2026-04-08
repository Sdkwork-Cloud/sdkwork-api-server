# 2026-04-07 Unreleased Step 06 Release External Dependency Coverage

## Summary

This unpublished release note records a release-path hardening slice that converts the current external sibling dependency surface into an executable contract.

The repository already materialized `sdkwork-ui` for release builds. This round adds a coverage audit proving that the current admin / portal release-app graph does not depend on any other unmanaged sibling repository.

This is not a published GitHub release and does not unblock release publication by itself.

## Decision Ledger

- Date: `2026-04-07`
- Status: `blocked / unpublished`
- Wave / Step: `B / 06`
- Primary mode: `release-gate-hardening`
- Previous mode: `release-gate-hardening`

### Top 3 Candidate Actions

1. Add a coverage audit that proves every current release-app sibling reference is backed by a declared materialization spec.
   - `Priority Score: 89`
2. Expand the materialization spec list to more sibling repositories without evidence.
   - `Priority Score: 54`
3. Leave the current dependency boundary undocumented and unverified.
   - `Priority Score: 37`

Action 1 was selected because it turns the current dependency boundary into a tested release fact.

## Delivered Changes

- updated `scripts/release/materialize-external-deps.mjs`
  - now scans release-app `package.json`, `pnpm-workspace.yaml`, and `tsconfig.json`
  - now blocks when any external sibling reference is not covered by declared materialization specs
- updated `scripts/release/release-workflow-contracts.mjs`
  - now asserts full coverage of the current external sibling dependency surface
- updated `scripts/release/tests/release-workflow.test.mjs`
  - now verifies the green coverage result and representative audited sources
- updated `docs/release/README.md`
  - documented the new coverage-audit rule
  - documented that the current release-app external sibling dependency surface is bounded to `sdkwork-ui`

## Verification Evidence

- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
- `node scripts/release/materialize-external-deps.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`

Observed result:

- coverage is complete
- current covered dependency ids: `sdkwork-ui`
- the helper remains non-destructive in the current workspace and reports:
  - `reused sdkwork-ui from Sdkwork-Cloud/sdkwork-ui@main`

## Release Window Impact

This slice improves release dependency truth and prevents unmanaged sibling drift, but the release window remains unpublished because:

- live repository sync truth is still blocked
- no verified commit / push / GitHub release path exists in the current session

This note remains part of `Unreleased` and must be merged into the next successfully verified GitHub release.
