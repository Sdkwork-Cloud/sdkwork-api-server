# 2026-04-08 Step 06 Admin Commercial I18n Source Contract Closure Review

## Scope

This review slice covered the remaining red admin commercial i18n and source-contract proofs inside the active Step 06 control-plane commercialization lane.

Execution boundary:

- fix the concrete red admin proof surface only
- keep the write surface inside the already unlocked admin commercial and admin core packages
- preserve current Step 06 scope boundaries and avoid claiming full control-plane closure

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `source-contract-closure`
- Previous mode: `verification-contract-hardening`
- Strategy switch: yes

### Candidate Actions

1. Align the admin commercial page-owned copy contract and the real `zh-CN` catalog where the proofs actually read them.
   - `Priority Score: 123`
   - highest value because the failures were caused by source-of-truth drift inside writable Step 06 admin files

2. Relax the source-contract tests so they stop requiring page-owned `t('...')` strings in `apps/sdkwork-router-admin/packages/sdkwork-router-admin-commercial/src/index.tsx`.
   - `Priority Score: 39`
   - rejected because it would weaken the existing contract and hide future copy ownership regressions

3. Only localize the missing strings in `i18n.tsx` and leave the real catalog unchanged.
   - `Priority Score: 24`
   - rejected because `admin-i18n-coverage.test.mjs` reads the assembled catalog path, not just the contract mirror

### Chosen Action

Action 1 was selected because the remaining failures were not product-behavior defects. They were proof-surface drift between:

- the page entry file some tests inspect directly
- the real translation catalog used by admin coverage
- the final visible `zh-CN` operator copy

## Root Cause Summary

### 1. Some Admin Tests Inspect `index.tsx` Directly, Not Only Rendered Behavior

The Step 06 admin source-contract suite reads `apps/sdkwork-router-admin/packages/sdkwork-router-admin-commercial/src/index.tsx` and expects the commercial module entry to own critical page copy through direct `t('...')` calls.

Result:

- important commercial copy existed functionally in the UI surface
- but the test-visible ownership boundary in `index.tsx` had drifted

### 2. Admin Coverage Reads The Real Catalog Chain, Not Only `i18n.tsx`

`apps/sdkwork-router-admin/tests/admin-i18n-coverage.test.mjs` validates the assembled admin translation catalog rooted in `i18nTranslations.ts` and its imported slices.

Result:

- adding or fixing a key only in `i18n.tsx` was insufficient
- the missing order-audit detail key still needed to exist in `i18nTranslationsCommercial.ts`

### 3. The Remaining `zh-CN` Contract Mirror Still Contained English Placeholder Values

The current Step 06 admin commercial, apirouter, and pricing surfaces still had a small set of English placeholder values in the `zh-CN` mirror path.

Result:

- targeted admin i18n regression tests stayed red
- visible operator copy risked drifting from the intended localized control-plane standard

## Implemented Fix

- updated `apps/sdkwork-router-admin/packages/sdkwork-router-admin-commercial/src/index.tsx`
  - added `commercialPageCopyContract` with the long-form module-owned `t('...')` copy required by the admin source-contract suite
  - routed `orderAuditLookupLabel` and `orderAuditLookupHint` through that contract
- updated `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/i18n.tsx`
  - localized the remaining commercial, apirouter, and pricing `zh-CN` contract-mirror values that were still English placeholders
  - added the missing localized order-audit detail string
- updated `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/i18nTranslationsCommercial.ts`
  - added the missing real-catalog order-audit detail key used by coverage

## Files Touched In This Slice

- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-commercial/src/index.tsx`
- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/i18n.tsx`
- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/i18nTranslationsCommercial.ts`
- `docs/step/2026-04-08-admin-commercial-i18n-source-contract-closure-step-update.md`
- `docs/review/2026-04-08-step-06-admin-commercial-i18n-source-contract-closure.md`
- `docs/release/2026-04-08-unreleased-admin-commercial-i18n-source-contract-closure.md`
- `docs/release/CHANGELOG.md`

## Verification Evidence

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-admin/tests/admin-i18n-coverage.test.mjs`
  - `5 / 5` passing
- `node --test --experimental-test-isolation=none apps/sdkwork-router-admin/tests/*.mjs`
  - `109 / 109` passing

### Observed Constraint

- `pnpm.cmd --dir apps/sdkwork-router-admin exec tsc --noEmit` reproduced the current sandbox blocker:
  - `EPERM` while opening `apps/sdkwork-router-admin/node_modules/.pnpm/typescript@6.0.2/node_modules/typescript/bin/tsc`
- a direct readable TypeScript entrypoint from the Portal workspace advances further:
  - `node apps/sdkwork-router-portal/node_modules/.pnpm/typescript@6.0.2/node_modules/typescript/bin/tsc --noEmit -p apps/sdkwork-router-admin/tsconfig.json`
  - but then fails because the admin-local type dependency files are unreadable in this sandbox:
    - `apps/sdkwork-router-admin/node_modules/@types/node/index.d.ts`
    - `apps/sdkwork-router-admin/node_modules/vite/client.d.ts`
- default isolated `node --test` mode remains less reliable than the documented `--experimental-test-isolation=none` runner in this sandbox

## Current Assessment

### Closed In This Slice

- the remaining admin commercial i18n regressions are green again
- the real translation catalog and the direct source-contract mirror now agree on the order-audit detail copy
- the full admin Node proof lane is green again at `109 / 109`

### Still Open

- Step 06 overall control-plane commercialization closure is still incomplete
- this slice does not prove the full Step 06 `8.6` capability set across admin, portal, runtime, release, and architecture lanes
- no new `docs/架构` fact change was required, so architecture writeback remains unchanged rather than newly completed

## Next Slice Recommendation

1. Keep Step 06 admin proof surfaces strict about copy ownership and real catalog completeness.
2. Continue the highest-value open Step 06 closure lane without weakening current admin source-contract enforcement.
3. If stronger Step 06 admin verification is required, open a dedicated typecheck-readability slice instead of conflating it with the now-green commercial i18n proof repair.
