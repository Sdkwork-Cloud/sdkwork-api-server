# 2026-04-08 Step 06 Admin Portal Test Contract Hardening Review

## Scope

This review slice covered the active Step 06 frontend verification failures in the admin and portal commercialization surfaces.

Execution boundary:

- fix concrete red frontend proofs only
- keep the write surface inside the already unlocked Step 06 admin/portal lane
- do not claim Step 06 closure or release completion

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `verification-contract-hardening`
- Previous mode: `verification-blocker-clearance`
- Strategy switch: yes

### Candidate Actions

1. Repair the active admin i18n coverage and portal redeem workspace contract failures before expanding the Step 06 scope.
   - `Priority Score: 111`
   - `S1` current-step closure push: `5 x 5 = 25`
   - `S2` Step 06 capability / `8.3` / `8.6` push: `4 x 5 = 20`
   - `S3` verification and release-gate push: `5 x 4 = 20`
   - `S4` blocker removal value: `4 x 4 = 16`
   - `S5` commercial delivery push: `3 x 3 = 9`
   - `S6` dual-runtime consistency value: `3 x 3 = 9`
   - `S7` immediate verifiability: `6 x 2 = 12`
   - `P1` churn / rework risk: `0 x -3 = 0`

2. Keep the current red frontend proofs and continue only on the release-truth lane.
   - `Priority Score: 54`
   - rejected because it would preserve known Step 06 red proofs inside the currently active admin/portal lane

3. Expand into broader portal or admin product work before stabilizing the verification surface.
   - `Priority Score: 61`
   - rejected because additional product work on top of unstable contracts would increase rework risk

### Chosen Action

Action 1 was selected because it removed concrete Step 06 proof failures with the smallest possible implementation surface and immediate red-to-green evidence.

## Root Cause Summary

### 1. Admin i18n Coverage Assumed a Readable App-Local TypeScript Runtime

The admin test imported `typescript` directly and assumed the app-local pnpm runtime was readable.

Result:

- in this Windows sandbox, the admin-local `typescript.js` existed but raised `EPERM` on open
- the test failed before any actual i18n coverage assertions could run

### 2. Portal Redeem Workspace Split Its Public Contract

Two active portal verification suites expected different layers of the redeem-history surface:

- the workspace-level polish contract expected `portal-redeem-history-table`
- the product-polish contract expected `portal-redeem-reward-history-table` plus the `Reward history` heading

Result:

- a single-slot implementation could satisfy only one proof at a time
- the redeem workspace drifted away from a stable layered contract

### 3. Portal History Summary Copy Bypassed Shared Localization

The credits page rendered the redeemed / rolled-back summary through a page-local inline string.

Result:

- the new summary sentence was not aligned with the shared portal translation inventory
- `zh-CN` would have fallen back to raw English for that operator-visible line

## Implemented Fixes

- updated the admin i18n coverage test to resolve pnpm package entries through a readable-entry helper
- replaced the previous readability check with a real `openSync` probe so unreadable files are rejected before import
- added a workspace fallback root so the admin test can use another readable pnpm-owned TypeScript runtime when the preferred local runtime is blocked by the sandbox
- restored the portal redeem workspace to a layered contract:
  - page wrapper: `portal-redeem-history-table`
  - detail table: `portal-redeem-reward-history-table`
- preserved the `Reward history` page heading while retaining `Redeem history` evidence copy for the higher-level workspace contract
- moved the redeemed / rolled-back summary onto shared portal i18n keys and added the matching `zh-CN` translation

## Files Touched In This Slice

- `apps/sdkwork-router-admin/tests/admin-i18n-coverage.test.mjs`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/pages/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts`

## Verification Evidence

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-admin/tests/admin-architecture.test.mjs apps/sdkwork-router-admin/tests/admin-commercial-workbench.test.mjs apps/sdkwork-router-admin/tests/admin-i18n-coverage.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-architecture.test.mjs apps/sdkwork-router-portal/tests/portal-auth-parity.test.mjs apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs apps/sdkwork-router-portal/tests/portal-admin-workspace-polish.test.mjs apps/sdkwork-router-portal/tests/portal-account-workbench.test.mjs apps/sdkwork-router-portal/tests/portal-account-history-i18n-polish.test.mjs apps/sdkwork-router-portal/tests/portal-account-posture-i18n-polish.test.mjs apps/sdkwork-router-portal/tests/portal-api-key-i18n.test.mjs`

### Observed Constraint

- default isolated `node --test` mode still fails with `spawn EPERM` in this sandbox
- the documented `--experimental-test-isolation=none` runner remains required for Step 06 frontend proof collection here

## Current Assessment

### Closed In This Slice

- admin i18n coverage proof is green again in the current pnpm workspace and sandbox
- portal redeem workspace contracts are green again across both workspace-level and product-level verification suites
- portal redeemed / rolled-back history summary now participates in shared localization instead of staying as inline-only copy

### Still Open

- Step 06 overall capability closure is still incomplete
- release-truth collection is still limited by the sandbox runner constraints and the lack of hosted proof
- the broader control-plane and commercialization acceptance gates in `91 / 95 / 97 / 98` still need continued execution beyond this recovery slice

## Maturity Delta

- `stateful standalone` fact maturity: unchanged
- `stateless runtime` fact maturity: unchanged
- Step 06 admin/portal frontend proof maturity: `L2 -> L3`

## Next Slice Recommendation

1. keep executing the highest-value open Step 06 closure lane now that admin/portal frontend verification is stable again
2. continue release-truth alignment without overstating sandbox-only verification as hosted fact
