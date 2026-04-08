# 2026-04-08 Unreleased Admin Typecheck Readability Recovery

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `blocker-clearance`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Introduce a repo-owned readable TypeScript launcher for admin and replace unreadable admin-local type roots with workspace-owned shims and declarations.
2. Downgrade or reshuffle admin-local frontend dependencies until raw `pnpm exec tsc` becomes readable again.
3. Leave admin typecheck blocked and rely on the existing admin Node proof lane as the de facto verification surface.

Action `1` was selected because the blocker was caused by unreadable app-local package files and stale frontend-tooling ownership boundaries, not by a need to weaken verification.

## 3. Actual Changes

- added `scripts/dev/run-tsc-cli.mjs`
  - the repository now owns a readable TypeScript execution entrypoint for frontend typecheck in the same style as `run-vite-cli.mjs`
- added `scripts/dev/vite-runtime-lib.d.mts`
  - repo-owned consumers now have a real declaration surface for the readable Vite runtime helper
- updated `apps/sdkwork-router-admin/package.json`
  - switched `typecheck` to `node ../../scripts/dev/run-tsc-cli.mjs --noEmit`
- updated `apps/sdkwork-router-admin/tsconfig.json` and `src/vite-env.d.ts`
  - removed direct reliance on unreadable admin-local `node` / `vite/client` type package entries
  - pointed the admin root at repository-owned readable shims and sibling UI declaration paths
- added admin-local readable type shims for:
  - Node runtime usage
  - Vite client globals
  - `lucide-react`
  - `react-router-dom`
  - `@sdkwork/ui-pc-react`
- repaired source typing defects exposed once typecheck could reach real admin code:
  - reused transport `putJson(...)` for `updateApiKey(...)`
  - restored `CommercialLatestSettlementsRailProps`
  - widened translation interpolation values to the real runtime usage
  - typed the admin Vite plugin loader and proxy rewrite callback
- updated verification contracts in:
  - `apps/sdkwork-router-admin/tests/admin-architecture.test.mjs`
  - `scripts/dev/tests/vite-runtime-lib.test.mjs`
  - `scripts/release-flow-contract.test.mjs`

## 4. Verification

- `pnpm.cmd --dir apps/sdkwork-router-admin run typecheck`
  - passing
- `node --test --experimental-test-isolation=none apps/sdkwork-router-admin/tests/*.mjs`
  - `110 / 110` passing
- `node --test --experimental-test-isolation=none scripts/release-flow-contract.test.mjs scripts/dev/tests/vite-runtime-lib.test.mjs`
  - `18 / 18` passing
- `pnpm.cmd --dir apps/sdkwork-router-admin exec tsc --noEmit`
  - still fails with `EPERM` while opening the unreadable app-local pnpm TypeScript bin shim

## 5. Architecture / Delivery Impact

- the admin frontend now has a repository-owned readable typecheck contract instead of a sandbox-fragile dependency on unreadable app-local pnpm binaries
- admin type roots are now explicitly controlled inside the repository boundary, which makes the verification path reproducible even when the app-local package layout is unreadable
- release-flow contract verification now reflects the real repo-owned frontend tooling pattern instead of a stale raw `vite` / `tsc` expectation
- no additional `docs/架构/*` writeback was required because this slice restores tooling ownership and verification truth, not a new operator-facing platform capability

## 6. Risks / Limits

- raw `pnpm exec tsc` remains blocked in this sandbox because the admin-local pnpm TypeScript bin shim is unreadable
- the new admin shims are intentionally narrow and should be retired if app-local package readability is later restored and can be proven
- Portal and other frontend packages still use their existing typecheck contract paths; broader wrapper standardization remains a separate decision
- Step 06 overall `8.3 / 8.6 / 91 / 95 / 97 / 98` closure remains open beyond this admin blocker-recovery slice

## 7. Next Entry

1. Keep the admin typecheck contract anchored to `run-tsc-cli.mjs`.
2. Decide separately whether the repo should standardize `run-tsc-cli.mjs` across Portal and any remaining frontend apps.
3. Continue the next highest-value Step 06 closure or blocker-removal lane without overstating this slice as full Step 06 completion.
