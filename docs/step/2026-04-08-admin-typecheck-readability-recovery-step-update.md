# 2026-04-08 Admin Typecheck Readability Recovery Step Update

## Slice Goal

Close the active Step 06 admin typecheck/readability blocker by routing admin typecheck through a repo-owned readable TypeScript entrypoint and replacing unreadable admin-local type dependencies with workspace-owned shims and declarations, without overstating broader Step 06 closure.

## Closed In This Slice

- added `scripts/dev/run-tsc-cli.mjs` so admin typecheck no longer depends on the unreadable app-local `typescript@6.0.2` binary entrypoint in the current Windows sandbox
- switched `apps/sdkwork-router-admin/package.json` `typecheck` to the repo-owned readable launcher instead of raw `tsc --noEmit`
- removed the admin root `tsconfig.json` dependency on unreadable local `"types": ["node", "vite/client"]`
- added admin-local readable shim surfaces:
  - `apps/sdkwork-router-admin/src/types/node-runtime-shim.d.ts`
  - `apps/sdkwork-router-admin/src/types/vite-client-shim.d.ts`
  - `apps/sdkwork-router-admin/src/types/lucide-react-shim.d.ts`
  - `apps/sdkwork-router-admin/src/types/react-router-dom-shim.d.ts`
  - `apps/sdkwork-router-admin/src/types/sdkwork-ui-pc-react-shim.d.ts`
- added `scripts/dev/vite-runtime-lib.d.mts` so the admin Vite config can typecheck against the repo-owned readable runtime helper instead of falling back to implicit `any`
- repaired latent admin source typing defects that only became visible after typecheck advanced past the unreadable dependency barrier:
  - reused transport `putJson(...)` in `sdkwork-router-admin-admin-api`
  - restored the missing `CommercialLatestSettlementsRailProps` type
  - widened admin translation interpolation values to the real runtime contract
  - annotated the admin Vite proxy rewrite callback and plugin factory loader path

## Runtime / Delivery Truth

### Admin Typecheck Contract Is Now Green Through The Repo-Owned Entry

- `pnpm.cmd --dir apps/sdkwork-router-admin run typecheck` now passes
- the admin root no longer requires unreadable app-local `@types/node` or `vite/client` package entries to satisfy its TypeScript contract
- the admin frontend tooling contract is now consistent with the already-established repo-owned `run-vite-cli.mjs` pattern: the repository, not the unreadable app-local package bin, owns the stable execution entrypoint

### Raw `pnpm exec tsc` Remains A Sandbox-Limited Path

- `pnpm.cmd --dir apps/sdkwork-router-admin exec tsc --noEmit` still fails in this sandbox with `EPERM` while opening:
  - `apps/sdkwork-router-admin/node_modules/.pnpm/typescript@6.0.2/node_modules/typescript/bin/tsc`
- this is now an observed raw-entry limitation, not the project contract path
- the repository contract for admin typecheck is the package script entry, not the unreadable pnpm bin shim

### Step 06 Scope Remains Bounded

- this slice closes the Step 06 admin typecheck/readability lane only
- it does not claim full Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` closure
- no new product capability or cross-lane architecture milestone was introduced, so no additional `docs/架构/*` writeback was required in this slice

## Verification

- `pnpm.cmd --dir apps/sdkwork-router-admin run typecheck`
  - passing
- `node --test --experimental-test-isolation=none apps/sdkwork-router-admin/tests/*.mjs`
  - `110 / 110` passing
- `node --test --experimental-test-isolation=none scripts/release-flow-contract.test.mjs scripts/dev/tests/vite-runtime-lib.test.mjs`
  - `18 / 18` passing

## Remaining Follow-Up

1. Keep admin typecheck on the repo-owned readable launcher unless app-local package readability can be proven again.
2. Decide separately whether Portal and other frontend packages should also standardize on `run-tsc-cli.mjs`; this slice intentionally avoided broadening that contract change beyond the blocked admin lane.
3. Continue the next highest-value Step 06 blocker or commercialization lane without claiming that the now-green admin typecheck slice closes Step 06 overall.
