# 2026-04-08 Unreleased Portal Billing Payment History Refund Status Productization

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `payment-history-refund-status-productization`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Productize the visible payment-history description so the tenant-facing Portal billing surface says `refund status` instead of `refund closure`.
2. Leave the old `refund closure` wording visible because the runtime still consumes canonical payment and refund evidence.
3. Broaden the slice into payment-history contracts or billing repository changes.

Action `1` was selected because it closes a visible commercialization gap immediately without widening the slice into runtime or architecture changes.

## 3. Actual Changes

- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
  - replaced the payment-history panel description with `Payment history keeps checkout outcomes, payment method evidence, and refund status visible in one billing timeline.`
  - removed the older `refund closure` wording from the tenant-facing billing page
- updated shared Portal i18n and `zh-CN` translations so the new payment-history description is fully registered
- updated Portal billing, product, and commercial-api proof so the new payment-history description is part of the active source contract

## 4. Verification

- red first:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- green:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
    - full Portal Node suite returned `221 / 221` passing

## 5. Architecture / Delivery Impact

- Portal billing payment-history now uses `refund status` product wording where it previously exposed `refund closure`
- the slice keeps payment-history payloads, billing repository composition, and runtime finance behavior unchanged
- no `docs/é‹èˆµç€¯/*` writeback was required because this iteration changed presentation copy, i18n, and proof only

## 6. Risks / Limits

- other low-level billing wording may still need future productization
- this slice changes product copy, i18n coverage, and proof only; it does not modify backend payment or refund contracts

## 7. Next Entry

1. Continue reducing remaining low-level billing wording where it still leaks into tenant-facing finance surfaces.
2. Keep future billing presentation fixes tied to the real formal backend billing path.
3. Continue the Step 06 commercialization lane from the now more productized Portal billing finance baseline.
