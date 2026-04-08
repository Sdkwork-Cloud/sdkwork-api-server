# 2026-04-08 Step 06 Admin Typecheck Readability Recovery Review

## Scope

This review slice covered the active Step 06 admin typecheck/readability blocker inside the control-plane commercialization lane.

Execution boundary:

- restore a real green admin typecheck contract
- keep the write surface focused on admin frontend tooling, admin-local type boundaries, and the newly surfaced admin source typing defects
- avoid broad cross-app TypeScript contract churn unless the admin fix required it

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `blocker-clearance`
- Previous mode: `source-contract-closure`
- Strategy switch: yes

### Candidate Actions

1. Introduce a repo-owned readable TypeScript launcher for admin and replace unreadable admin-local type dependencies with workspace-owned shims/declarations.
   - `Priority Score: 131`
   - highest value because the blocker was caused by unreadable local package files, not by an actual product or business-logic defect

2. Downgrade or reshuffle admin-local TypeScript and frontend dependencies until `pnpm exec tsc` becomes readable again.
   - `Priority Score: 41`
   - rejected because the current workspace is already dirty, the unreadability appears filesystem-layout specific, and that path would still leave the repository without a stable owned entrypoint

3. Treat the admin Node proof lane as sufficient and defer typecheck entirely.
   - `Priority Score: 18`
   - rejected because it would leave a real Step 06 verification hole in the admin frontend contract

### Chosen Action

Action 1 was selected because the root blocker was not semantic TypeScript failure. It was that the repository delegated execution and type roots to unreadable app-local package files.

## Root Cause Summary

### 1. The Admin App-Local TypeScript Entry Was Unreadable

Fresh evidence showed:

- `pnpm.cmd --dir apps/sdkwork-router-admin exec tsc --noEmit`
  - failed with `EPERM` while opening `apps/sdkwork-router-admin/node_modules/.pnpm/typescript@6.0.2/node_modules/typescript/bin/tsc`
- the matching admin-local `lib/tsc.js` and `lib/typescript.js` files were also unreadable

Result:

- raw `pnpm exec tsc` could not even start the compiler
- the repository had no owned frontend typecheck entrypoint for this admin app

### 2. The Admin Root Also Pointed TypeScript At Unreadable App-Local Type Libraries

Fresh evidence showed:

- `apps/sdkwork-router-admin/node_modules/@types/node/index.d.ts` was unreadable
- `apps/sdkwork-router-admin/node_modules/vite/client.d.ts` was unreadable
- the admin root `tsconfig.json` explicitly required both through `"types": ["node", "vite/client"]`

Result:

- even when a readable donor TypeScript runtime was used, typecheck still failed before reaching real source validation

### 3. Existing Release Contract Tests Were Stale Relative To Real Frontend Tooling Ownership

`scripts/release-flow-contract.test.mjs` still expected raw `vite`/`tsc` shell contracts even though the repository already used `run-vite-cli.mjs` for browser-mode ownership.

Result:

- release-flow contract verification no longer matched the actual repository-owned frontend tooling boundary

### 4. Once Typecheck Advanced, Real Admin Source Typing Defects Surfaced

After the readability barrier was removed, admin typecheck exposed actual source issues:

- `updateApiKey(...)` in `sdkwork-router-admin-admin-api` depended on non-exported transport helpers
- `CommercialLatestSettlementsRailProps` was missing
- admin translation interpolation types were narrower than the real downstream helper contracts
- the admin Vite config relied on undeclared external helper types and an unannotated rewrite callback

Result:

- the readability fix alone was insufficient; a second pass was required to close real source typing gaps

## Implemented Fix

- added `scripts/dev/run-tsc-cli.mjs`
  - resolves a readable `typescript/lib/tsc.js` through the same donor-root strategy already used by `run-vite-cli.mjs`
- added `scripts/dev/vite-runtime-lib.d.mts`
  - gives repo-owned consumers a real declaration surface for the readable Vite runtime helper
- updated `apps/sdkwork-router-admin/package.json`
  - switched `typecheck` to `node ../../scripts/dev/run-tsc-cli.mjs --noEmit`
- updated `apps/sdkwork-router-admin/tsconfig.json`
  - removed unreadable local `"types": ["node", "vite/client"]`
  - routed `@sdkwork/ui-pc-react` root typing through a local shim and readable sibling dist paths
- updated `apps/sdkwork-router-admin/src/vite-env.d.ts`
  - pointed the admin root at repo-owned readable shim files
- added admin-local readable shims:
  - `node-runtime-shim.d.ts`
  - `vite-client-shim.d.ts`
  - `lucide-react-shim.d.ts`
  - `react-router-dom-shim.d.ts`
  - `sdkwork-ui-pc-react-shim.d.ts`
- fixed newly exposed source typing defects:
  - replaced raw `fetch` + non-exported helper usage with `putJson(...)` in `sdkwork-router-admin-admin-api/src/index.ts`
  - restored `CommercialLatestSettlementsRailProps` in `sdkwork-router-admin-commercial/src/commercialOverviewSections.tsx`
  - widened translation interpolation values in `sdkwork-router-admin-core/src/i18n.tsx`
  - annotated the admin Vite proxy rewrite callback and plugin loader result types in `apps/sdkwork-router-admin/vite.config.ts`
- updated verification contracts:
  - `apps/sdkwork-router-admin/tests/admin-architecture.test.mjs`
  - `scripts/dev/tests/vite-runtime-lib.test.mjs`
  - `scripts/release-flow-contract.test.mjs`

## Files Touched In This Slice

- `apps/sdkwork-router-admin/package.json`
- `apps/sdkwork-router-admin/tsconfig.json`
- `apps/sdkwork-router-admin/src/vite-env.d.ts`
- `apps/sdkwork-router-admin/src/types/node-runtime-shim.d.ts`
- `apps/sdkwork-router-admin/src/types/vite-client-shim.d.ts`
- `apps/sdkwork-router-admin/src/types/lucide-react-shim.d.ts`
- `apps/sdkwork-router-admin/src/types/react-router-dom-shim.d.ts`
- `apps/sdkwork-router-admin/src/types/sdkwork-ui-pc-react-shim.d.ts`
- `apps/sdkwork-router-admin/src/types/vite-runtime-lib-shim.d.ts`
- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts`
- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-commercial/src/commercialOverviewSections.tsx`
- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/i18n.tsx`
- `apps/sdkwork-router-admin/vite.config.ts`
- `apps/sdkwork-router-admin/tests/admin-architecture.test.mjs`
- `scripts/dev/run-tsc-cli.mjs`
- `scripts/dev/vite-runtime-lib.d.mts`
- `scripts/dev/tests/vite-runtime-lib.test.mjs`
- `scripts/release-flow-contract.test.mjs`
- `docs/step/2026-04-08-admin-typecheck-readability-recovery-step-update.md`
- `docs/review/2026-04-08-step-06-admin-typecheck-readability-recovery.md`
- `docs/release/2026-04-08-unreleased-admin-typecheck-readability-recovery.md`
- `docs/release/CHANGELOG.md`

## Verification Evidence

### Green

- `pnpm.cmd --dir apps/sdkwork-router-admin run typecheck`
  - passing
- `node --test --experimental-test-isolation=none apps/sdkwork-router-admin/tests/*.mjs`
  - `110 / 110` passing
- `node --test --experimental-test-isolation=none scripts/release-flow-contract.test.mjs scripts/dev/tests/vite-runtime-lib.test.mjs`
  - `18 / 18` passing

### Observed Constraint

- `pnpm.cmd --dir apps/sdkwork-router-admin exec tsc --noEmit`
  - still fails with `EPERM` while opening the unreadable app-local pnpm TypeScript bin shim
- this slice intentionally fixes the repository contract path, not the raw pnpm bin readability of the sandboxed filesystem

## Current Assessment

### Closed In This Slice

- the active Step 06 admin typecheck/readability lane is green again through the repository-owned entrypoint
- admin frontend type boundaries are now controlled by repository shims/declarations instead of unreadable app-local package files
- the stale release-flow script contract now matches the real repo-owned frontend tooling boundary

### Still Open

- raw `pnpm exec tsc` remains unreadable in this sandbox
- Portal and other frontend packages have not been migrated to `run-tsc-cli.mjs`; that broader standardization remains an optional future tooling slice
- Step 06 overall closure remains open beyond this admin blocker-removal slice
- no new `docs/架构/*` writeback was required because this slice aligns an existing frontend-tooling ownership pattern rather than adding a new platform capability

## Next Slice Recommendation

1. Keep admin typecheck on the repo-owned readable launcher and avoid reintroducing unreadable app-local type roots.
2. If cross-app frontend tooling standardization becomes valuable, open a dedicated wrapper-standardization slice for Portal and any remaining frontend packages.
3. Continue the next highest-value Step 06 blocker or commercialization lane without claiming Step 06 completion from this admin recovery alone.
