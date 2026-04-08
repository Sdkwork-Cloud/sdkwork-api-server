# 2026-04-08 Unreleased Portal Billing Sandbox Surface Vocabulary Productization

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `sandbox-surface-vocabulary-productization`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Productize the remaining billing sandbox title, selector, and verification labels so the visible workbench speaks in payment-outcome, sandbox-method, and verification language.
2. Leave the current `event` / `target` / `signature` wording in place because the sandbox remains a technical surface.
3. Broaden the slice into runtime callback-contract renames.

Action `1` was selected because it closes a visible commercialization gap immediately without widening the slice into runtime or architecture changes.

## 3. Actual Changes

- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
  - renamed `Payment event sandbox` to `Payment outcome sandbox`
  - renamed `Event target` / `Choose event target` to `Sandbox method` / `Choose sandbox method`
  - renamed `Event signature` to `Verification method`
  - rewrote the active sandbox sentence to `Payment outcomes will use {provider} on {channel}.`
- updated shared Portal i18n and `zh-CN` translations so the new sandbox-surface wording is fully registered
- updated Portal billing and payment-rails proof so the old sandbox-surface wording now fails source-contract verification

## 4. Verification

- red first:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
- green:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
  - `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
    - full Portal Node suite returned `221 / 221` passing

## 5. Architecture / Delivery Impact

- Portal billing sandbox now presents title, selection, and verification posture through product-facing outcome and method language instead of low-level event/target/signature jargon
- the slice keeps all runtime payment behavior unchanged while improving Step 06 commercialization evidence
- no `docs/架构/*` writeback was required because this iteration changed presentation copy, i18n, and proof only

## 6. Risks / Limits

- raw `webhook_verification` values still render directly and may still need a future product-display mapping
- sandbox posture remains intentionally visible because the billing workbench still includes payment simulation tooling
- this slice changes product copy, i18n coverage, and proof only; it does not modify backend payment contracts or billing behavior

## 7. Next Entry

1. Continue reducing remaining low-level billing metadata wording where it still leaks into the tenant-facing sandbox surface.
2. Keep future billing presentation fixes tied to the real formal backend payment path.
3. Continue the Step 06 commercialization lane from the now more productized sandbox-surface baseline.
