# 2026-04-07 Step 06 Release Dependency Materialization Review

## Scope

This review slice continued Wave `B` / Step `06` after the release-blocker audit and targeted one concrete, high-leverage gap on the release path:

- release builds still assumed the local sibling `sdkwork-ui` repository already existed after checkout

Execution boundary for this slice:

- keep local development on the existing relative-path / workspace dependency model
- add the minimum release-only infrastructure needed to materialize external sibling dependencies from GitHub
- avoid changing application behavior, package ownership, or the local workspace layout

## Decision Ledger

- Date: `2026-04-07`
- Wave / Step: `B / 06`
- Primary mode: `release-gate-hardening`
- Previous mode: `release-gate-truthfulness`
- Strategy switch: yes

### Top 3 Candidate Actions

1. Add a tested release-only materialization step that clones the required GitHub sibling dependency into the path expected by the existing admin / portal workspace graph.
   - `Priority Score: 88`
   - `S1` current-step closure push: `3 x 5 = 15`
   - `S2` Step 06 capability / `8.3` / `8.6` / `95` push: `3 x 5 = 15`
   - `S3` verification and release-gate push: `5 x 4 = 20`
   - `S4` blocker removal value: `4 x 4 = 16`
   - `S5` commercial delivery push: `3 x 3 = 9`
   - `S6` dual-runtime consistency value: `1 x 3 = 3`
   - `S7` immediate verifiability: `5 x 2 = 10`
   - `P1` churn / rework risk: `0 x -3 = 0`

2. Attempt mainline commit / push after the blocker audit without fixing the CI dependency gap.
   - `Priority Score: 31`
   - rejected because it would still leave the release workflow structurally unable to reproduce the local dependency graph

3. Start a broad Step 06 capability audit across the entire staged snapshot before fixing the release-environment dependency model.
   - `Priority Score: 66`
   - useful, but lower leverage than closing a concrete release blocker that could invalidate any later release attempt

### Chosen Action

Action 1 was selected because it removes one real release-path blocker with low write-surface expansion and produces direct, fresh verification evidence without disturbing the local relative-path SDK workflow.

## Problem Summary

Current admin and portal dependency wiring still relies on a sibling `sdkwork-ui` repository:

- `apps/sdkwork-router-admin/package.json` uses `file:..\\..\\..\\sdkwork-ui\\sdkwork-ui-pc-react`
- `apps/sdkwork-router-portal/package.json` uses `workspace:*`
- `apps/sdkwork-router-portal/pnpm-workspace.yaml` includes `../../../sdkwork-ui/sdkwork-ui-pc-react`

But the GitHub release workflow previously:

- checked out only `sdkwork-api-router`
- ran `pnpm install` directly for admin and portal

Result:

- the release environment did not reproduce the local workspace graph
- release builds depended on an unspoken external repository layout

## Implemented Fixes

- added `scripts/release/materialize-external-deps.mjs`
  - declares the release-only external dependency spec for `sdkwork-ui`
  - clones `https://github.com/Sdkwork-Cloud/sdkwork-ui.git` into the sibling path expected by the current package graph
  - supports explicit ref control through `SDKWORK_UI_GIT_REF`
  - reuses an already-present valid sibling checkout without mutating local workspace state
  - fails fast if the sibling path exists but is incomplete
- updated `.github/workflows/release.yml`
  - added a `Materialize external release dependencies` step to both `native-release` and `web-release`
  - placed the step after Node setup and before any frontend `pnpm install`
  - forwarded `SDKWORK_UI_GIT_REF` with a default of `main`
- added release workflow contract coverage for the new helper and workflow ordering

## Files Touched In This Slice

- `.github/workflows/release.yml`
- `scripts/release/materialize-external-deps.mjs`
- `scripts/release/tests/release-workflow.test.mjs`

## Verification Evidence

### Red

- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
  - initially failed because the workflow did not contain a materialization step and the helper script did not exist

### Green

- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
- `node scripts/release/materialize-external-deps.mjs`
  - local result: `reused sdkwork-ui from Sdkwork-Cloud/sdkwork-ui@main`

## Current Assessment

### Closed In This Slice

- the release workflow now has an explicit, tested way to materialize the `sdkwork-ui` sibling dependency from GitHub
- local relative-path dependency design remains intact
- the release dependency model is no longer implicit or undocumented

### Still Open

- remote verification for `sdkwork-api-router` is still required before any push or release
- `sdkwork-core`, `sdkwork-ui`, `sdkwork-appbase`, and `sdkwork-im-sdk` still need clean synchronized release truth before a real release attempt
- the workflow currently materializes only the external sibling actually required by this repository's frontend install graph; broader cross-repository sync verification is still a separate release gate
- Step `06` overall closure remains incomplete

## Maturity Delta

- `stateful standalone` fact maturity: unchanged at `L3`
- `stateless runtime` fact maturity: unchanged at `L2`

This slice improved release reproducibility rather than product-surface capability maturity.

## Next Slice Recommendation

1. Re-verify remote GitHub access and dependent repository cleanliness before any mainline commit / push attempt.
2. If remote truth becomes available, pin `SDKWORK_UI_GIT_REF` to the verified synchronized GitHub ref instead of relying on the default branch.
3. Continue Step `06` release gating by aligning broader dependency sync truth, mainline snapshot strategy, and final unpublished release-window consolidation.
