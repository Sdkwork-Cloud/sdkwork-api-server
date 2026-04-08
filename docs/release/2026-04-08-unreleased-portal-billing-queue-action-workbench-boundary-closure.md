# 2026-04-08 Unreleased Portal Billing Queue Action Workbench Boundary Closure

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `queue-action-workbench-boundary-closure`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Remove queue-row `settle` / `cancel` ownership and keep those actions only inside the opened checkout workbench.
2. Delete the remaining explicit bridge/manual actions from Portal billing entirely.
3. Leave queue-row actions in place and rely on softer copy only.

Action `1` was selected because it closes a visible commercialization gap immediately, without changing backend contracts or deleting runtime-supported explicit actions prematurely.

## 3. Actual Changes

- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
  - removed queue-row `Settle order` and `Cancel order` actions from the default `Pending payment queue`
  - kept those actions inside the opened checkout workbench method cards through `activeCheckoutOrder`
  - updated the post-order status guidance to send users to the checkout workbench before quota or membership changes are applied
- updated shared Portal i18n registries and `zh-CN` translations for the new queue guidance copy
- updated billing source-contract tests so the workbench boundary and queue guidance stay protected

## 4. Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## 5. Architecture / Delivery Impact

- the Portal billing queue now behaves more like a productized pending-order inventory and less like a default settlement console
- explicit bridge/manual actions remain available, but only inside the opened checkout workbench where order and rail context already exist
- the slice improves Step 06 commercialization evidence without overstating whole-step completion

## 6. Risks / Limits

- explicit bridge/manual actions still remain in the checkout workbench
- payment simulation posture still remains part of the billing runtime
- compatibility checkout-session still exists as a bridge / fallback source
- this slice does not add new backend payment contracts or remove the remaining compatibility mutation paths

## 7. Next Entry

1. Continue reducing compatibility bridge ownership inside the default billing payment journey where formal backend truth already exists.
2. Keep future Portal billing changes tied to real backend semantics only.
3. Continue the Step 06 commercialization lane from the now tighter queue-versus-workbench payment boundary.
