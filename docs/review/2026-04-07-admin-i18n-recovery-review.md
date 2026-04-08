# 2026-04-07 Admin I18n Recovery Review

## Scope

This review slice executed the current Step 06 control-plane loop against the real workspace state, with a narrow focus on the admin translation load path and the verification gates that block commercial-grade admin delivery.

Validated surfaces in this round:

- `apps/sdkwork-router-admin/tests/admin-commercial-api-surface.test.mjs`
- `apps/sdkwork-router-admin/tests/admin-commercial-workbench.test.mjs`
- `apps/sdkwork-router-admin/tests/admin-i18n-coverage.test.mjs`
- `scripts/check-rust-verification-matrix.mjs --group admin-service`
- `scripts/check-rust-verification-matrix.mjs --group portal-service`

## What Was Fixed

1. Repaired the stale admin commercial workbench regression so it validates the current package-split admin implementation instead of assuming the old monolithic source layout.
2. Reworked the admin i18n coverage test so it loads the real translation module graph rather than assuming `ADMIN_TRANSLATIONS` is declared inline in `i18n.tsx`.
3. Normalized the corrupted admin translation source chain:
   - `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/i18nTranslations.ts`
   - `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/i18nTranslationsCore.ts`
   - `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/i18nTranslationsCommercial.ts`
   - `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/i18nTranslationsRouting.ts`
4. Converted the broken translation tables into stable ASCII-escaped TypeScript objects so the module graph is loadable again under `jiti` and test/runtime consumers no longer fail on unterminated string constants.
5. Added `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/i18nTranslationsRecovery.ts` and wired it into the root admin catalog to backfill 158 newly used payment, webhook, refund, and reconciliation keys that were present in source but absent from the `zh-CN` catalog.

## Verification Evidence

### Green

- `node apps/sdkwork-router-admin/tests/admin-commercial-api-surface.test.mjs`
- `node apps/sdkwork-router-admin/tests/admin-commercial-workbench.test.mjs`
- `node apps/sdkwork-router-admin/tests/admin-i18n-coverage.test.mjs`

The admin control-plane translation gate is green again, and the commercial workspace structural tests remain green after the translation-source recovery.

### Red

- `node scripts/check-rust-verification-matrix.mjs --group admin-service`
- `node scripts/check-rust-verification-matrix.mjs --group portal-service`

Both Rust verification groups are currently blocked by pre-existing compile failures in `crates/sdkwork-api-domain-billing`, not by the admin i18n changes from this round.

Observed blocker details:

- `crates/sdkwork-api-domain-billing/src/accounts.rs:664`
  - `expected item after attributes`
- `crates/sdkwork-api-domain-billing/src/pricing.rs`
  - missing `PricingPlanId`
  - missing `AccountId`
  - missing `PricingRateId`

## Current Assessment

### Completed in this round

- The admin commercial workspace is no longer blocked by broken translation source syntax.
- The admin i18n structural gate now reflects the real codebase again.
- Newly introduced payment/control-plane UI keys are explicitly cataloged instead of silently escaping translation coverage.

### Still open

- The new recovery catalog intentionally uses English fallback values for the 158 newly cataloged payment/reconciliation keys. This is acceptable for structural recovery, but it is not a final localization state.
- The shared billing-domain crate remains the next critical blocker for Step 06 service-level verification.

## Recommended Next Slice

1. Fix the `sdkwork-api-domain-billing` compile breakages so `admin-service` and `portal-service` package-group checks can return to green.
2. Replace the generated English-valued recovery entries in `i18nTranslationsRecovery.ts` with curated Simplified Chinese translations in verified batches, starting with:
   - payment method management
   - webhook inbox and replay defense
   - refund operations
   - reconciliation runs and discrepancy ledger
3. Keep `admin-i18n-coverage.test.mjs` in the verification set for every admin payment/control-plane slice so the recovery catalog does not drift again.
