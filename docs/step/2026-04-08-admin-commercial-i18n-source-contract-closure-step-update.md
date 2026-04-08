# 2026-04-08 Admin Commercial I18n Source Contract Closure Step Update

## Slice Goal

Close the remaining Step 06 admin commercial i18n and source-contract failures by aligning the page-owned copy contract in the commercial module with the real `zh-CN` translation catalog, without overstating broader Step 06 closure.

## Closed In This Slice

- localized the remaining admin commercial, apirouter, and pricing `zh-CN` contract-mirror strings in `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/i18n.tsx`
- added a page-level `commercialPageCopyContract` to `apps/sdkwork-router-admin/packages/sdkwork-router-admin-commercial/src/index.tsx` so source-contract tests can read module-owned `t('...')` usage directly from the commercial page entry instead of relying on deeper component indirection
- routed the order-audit lookup label and hint through that page-level copy contract without changing the live commercial workspace behavior
- added the missing order-audit detail key to `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/i18nTranslationsCommercial.ts` so the real translation catalog used by coverage is consistent with the contract mirror

## Runtime / Display Truth

### Admin Commercial Copy Contract Is Now Aligned Across The Two Real Sources

- `i18n.tsx` remains the direct contract mirror inspected by a portion of the admin source-contract suite
- `i18nTranslationsCommercial.ts` remains the real catalog consumed by the assembled admin translation inventory and `admin-i18n-coverage.test.mjs`
- the selected-order audit hint now exists in both layers, which keeps visible operator copy, source-contract proofs, and catalog coverage in sync

### Step 06 Scope Remains Bounded

- this slice hardens the Step 06 admin commercial proof surface only
- it does not close Step 06 overall
- no new control-plane architecture capability was introduced beyond restoring proof and localization consistency, so no additional `docs/架构/133-*`, `139-*`, or `141-*` writeback was required in this slice

## Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-admin/tests/admin-i18n-coverage.test.mjs`
  - `5 / 5` passing
- `node --test --experimental-test-isolation=none apps/sdkwork-router-admin/tests/*.mjs`
  - `109 / 109` passing

## Verification Constraint

- `pnpm.cmd --dir apps/sdkwork-router-admin exec tsc --noEmit`
  - blocked by `EPERM` while opening `apps/sdkwork-router-admin/node_modules/.pnpm/typescript@6.0.2/node_modules/typescript/bin/tsc`
- `node apps/sdkwork-router-portal/node_modules/.pnpm/typescript@6.0.2/node_modules/typescript/bin/tsc --noEmit -p apps/sdkwork-router-admin/tsconfig.json`
  - advances past the unreadable `tsc` entrypoint
  - then fails because the admin-local type dependency files are also unreadable in this sandbox:
    - `apps/sdkwork-router-admin/node_modules/@types/node/index.d.ts`
    - `apps/sdkwork-router-admin/node_modules/vite/client.d.ts`

## Remaining Follow-Up

1. Keep Step 06 admin commercial proofs anchored to module-owned copy contracts and real catalog ownership instead of weakening the tests around those boundaries.
2. Continue Step 06 control-plane and commercialization closure without claiming that this proof slice alone satisfies the full `8.3 / 8.6 / 91 / 95 / 97 / 98` gate set.
3. Treat Admin typecheck as a separate sandbox-readability lane: the next blocker is local type dependency readability, not the commercial i18n proof surface.
