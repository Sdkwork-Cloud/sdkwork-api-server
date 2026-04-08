# 2026-04-07 Unreleased Step 06 Release Dependency Materialization

## Summary

This unpublished release note records a concrete release-path hardening slice inside Wave `B` / Step `06`.

The release workflow now has an explicit, tested mechanism to materialize the external sibling `sdkwork-ui` repository from GitHub before frontend dependency installation, while preserving the existing local relative-path dependency model for development.

This is not a published GitHub release. It remains part of the current `Unreleased` window because push / release eligibility is still blocked by remote verification and dependent-repository sync truth.

## Decision Ledger

- Date: `2026-04-07`
- Status: `blocked / unpublished`
- Wave / Step: `B / 06`
- Primary mode: `release-gate-hardening`
- Previous mode: `release-gate-truthfulness`

### Top 3 Candidate Actions

1. Add a tested release-only external dependency materialization step for the GitHub-backed sibling UI repository.
   - `Priority Score: 88`
2. Attempt commit / push without fixing the release dependency gap.
   - `Priority Score: 31`
3. Expand immediately into a broad Step 06 capability audit before hardening the release path.
   - `Priority Score: 66`

Action 1 was selected because it removes a real CI release blocker without disturbing the local workspace dependency model.

## Delivered Changes

- added `scripts/release/materialize-external-deps.mjs`
  - clones `Sdkwork-Cloud/sdkwork-ui` into the sibling path expected by the current admin / portal dependency graph
  - supports `SDKWORK_UI_GIT_REF`
  - reuses an already-valid local sibling checkout instead of mutating it
- updated `.github/workflows/release.yml`
  - both `native-release` and `web-release` now materialize external release dependencies before `pnpm install`
- extended `scripts/release/tests/release-workflow.test.mjs`
  - release workflow contracts now verify the helper exists and is wired before frontend installation

## Verification Evidence

- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
- `node scripts/release/materialize-external-deps.mjs`

Local helper behavior was non-destructive in the current workspace and reported:

- `reused sdkwork-ui from Sdkwork-Cloud/sdkwork-ui@main`

## Release Window Impact

This slice improves release reproducibility but does not yet authorize release publication.

The release window remains unpublished because:

- `sdkwork-api-router` remote verification is still unavailable in the current session
- dependent repository synchronization still has unresolved dirty / boundary issues

Therefore this note must remain part of `Unreleased` and be merged into the next successfully verified GitHub release.
