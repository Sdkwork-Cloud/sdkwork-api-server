# 2026-04-08 Unreleased Portal Billing Checkout Metadata Productization

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `checkout-metadata-productization`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Productize the remaining checkout metadata labels in Portal billing so the method cards read as commercial checkout guidance instead of operator/webhook terminology.
2. Remove the checkout metadata fields from the method cards entirely.
3. Leave the current labels in place and rely on technical accuracy alone.

Action `1` was selected because it closes a visible commercialization gap immediately without widening the slice into runtime or architecture changes.

## 3. Actual Changes

- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
  - renamed `Operator action` to `Manual action`
  - renamed `Webhook` to `Provider events`
  - renamed `Webhook verification` to `Event signature`
  - renamed `Refund support` to `Refund coverage`
  - renamed `Partial refund` to `Partial refunds`
- updated shared Portal i18n registries and `zh-CN` translations for the new checkout metadata vocabulary
- updated Portal payment-rails and billing-i18n source-contract tests so the old operator/webhook/refund-support wording now fails proof

## 4. Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## 5. Architecture / Delivery Impact

- checkout method cards now present payment metadata through product-facing billing language
- the slice keeps all runtime payment behavior unchanged while improving Step 06 commercialization evidence
- no `docs/架构/*` writeback was required because this iteration changed presentation copy, i18n, and proof only

## 6. Risks / Limits

- explicit payment-event sandbox behavior still remains in runtime
- manual settlement bridge behavior still remains when simulation posture permits it
- this slice changes product copy, i18n coverage, and proof only; it does not modify backend payment contracts or billing behavior

## 7. Next Entry

1. Continue reducing internal payment vocabulary from Portal billing.
2. Keep future billing presentation fixes tied to the real formal backend payment path.
3. Continue the Step 06 commercialization lane from the now more productized checkout metadata baseline.
