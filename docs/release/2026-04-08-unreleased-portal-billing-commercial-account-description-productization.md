# 2026-04-08 Unreleased Portal Billing Commercial Account Description Productization

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `commercial-account-description-productization`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Productize the visible commercial-account description so the tenant-facing Portal billing workbench stops saying `Commercial account exposes canonical balance, hold, and account identity state ...`.
2. Leave the old `canonical balance` wording visible because the runtime already loads the correct commercial-account facts.
3. Broaden the slice into commercial-account runtime or backend contract refactors.

Action `1` was selected because it closes a visible commercialization gap immediately without widening the slice into runtime or architecture changes.

## 3. Actual Changes

- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
  - replaced the commercial-account panel description with `Commercial account keeps balance, holds, and account identity visible beside the workspace billing posture.`
  - removed the older `Commercial account exposes canonical balance, hold, and account identity state ...` wording from the tenant-facing billing page
- updated shared Portal i18n and `zh-CN` translations so the new commercial-account description is fully registered
- updated Portal billing, product, and commercial-api proof so the new commercial-account description is part of the active source contract

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
    - full Portal Node suite returned `222 / 222` passing

## 5. Architecture / Delivery Impact

- Portal billing commercial-account copy now uses tenant-facing product language where it previously exposed `canonical balance` and `account identity state` implementation wording
- the slice keeps commercial-account payloads, repository composition, and runtime finance behavior unchanged
- no `docs/架构/*` writeback was required because this iteration changed presentation copy, i18n, and proof only

## 6. Risks / Limits

- other low-level billing wording may still need future productization
- this slice changes product copy, i18n coverage, and proof only; it does not modify backend commerce contracts

## 7. Next Entry

1. Continue reducing remaining low-level billing wording where it still leaks into tenant-facing finance surfaces.
2. Keep future billing presentation fixes tied to the real backend checkout flow without renaming backend fields or payloads.
3. Continue the Step 06 commercialization lane from the now more productized Portal billing baseline.
