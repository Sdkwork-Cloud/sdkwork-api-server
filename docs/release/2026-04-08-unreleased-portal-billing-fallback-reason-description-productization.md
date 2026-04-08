# 2026-04-08 Unreleased Portal Billing Fallback Reason Description Productization

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `fallback-reason-description-productization`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Productize the visible fallback-reason description so the tenant-facing Portal billing surface explains degraded routing from the user viewpoint rather than the operator viewpoint.
2. Leave the old operator-facing wording visible because the runtime already loads the correct fallback evidence.
3. Broaden the slice into routing analytics contracts or billing repository changes.

Action `1` was selected because it closes a visible commercialization gap immediately without widening the slice into runtime or architecture changes.

## 3. Actual Changes

- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
  - replaced the fallback-reason description with `Fallback reasoning stays visible so you can distinguish degraded routing from the preferred routing path.`
  - removed the older operator-facing wording from the tenant-facing billing page
- updated shared Portal i18n and `zh-CN` translations so the new fallback-reason description is fully registered
- updated Portal billing, product, and commercial-api proof so the new fallback-reason description is part of the active source contract

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

- Portal billing fallback guidance now uses user-facing routing language where it previously exposed an operator viewpoint
- the slice keeps billing event analytics payloads, repository composition, and runtime finance behavior unchanged
- no `docs/é‹èˆµç€¯/*` writeback was required because this iteration changed presentation copy, i18n, and proof only

## 6. Risks / Limits

- other low-level billing wording may still need future productization
- this slice changes product copy, i18n coverage, and proof only; it does not modify routing or billing backend contracts

## 7. Next Entry

1. Continue reducing remaining low-level billing wording where it still leaks into tenant-facing finance surfaces.
2. Keep future billing presentation fixes tied to the real formal backend billing path.
3. Continue the Step 06 commercialization lane from the now more productized Portal billing finance baseline.
