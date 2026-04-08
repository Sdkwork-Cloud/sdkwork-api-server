# 2026-04-08 Portal Billing Settlement And Sandbox Posture Productization Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by productizing the remaining settlement-summary and payment-sandbox copy in the billing workspace, so the default tenant surface stops describing those areas through operator-facing or callback-rehearsal language.

## Closed In This Slice

- renamed the billing summary panel from `Commercial settlement rail` to `Settlement coverage`
- replaced the settlement summary description so it now explains benefit lots, credit holds, and request capture as one billing snapshot instead of an operator-facing posture
- renamed the explicit simulation panel from `Provider callbacks` to `Payment event sandbox`
- rewrote the sandbox description, selector label, placeholder, status sentence, and button labels so the surface now reads as a sandbox payment-event tool instead of a callback rehearsal console
- extended shared Portal i18n and `zh-CN` coverage for the new settlement/sandbox product wording
- updated product, workspace, and billing i18n source-contract tests to require the new copy and reject the retired operator/callback language

## Runtime / Display Truth

### Settlement Summary Now Reads As Billing Product Copy

- the billing workspace still shows benefit lots, credit holds, and request settlement capture
- only the label and description changed in this slice
- no commercial account calculation, order mutation, or payment flow behavior changed

### The Explicit Simulation Surface Still Exists, But Its Boundary Is Clearer

- the simulation panel still remains gated by `paymentSimulationEnabled`
- the selected provider rail, settlement/failure/cancel actions, and runtime mutation path are unchanged
- the user-facing copy now describes the surface as a `Payment event sandbox` with `Sandbox only` posture instead of a callback rehearsal baseline

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by removing more operator-oriented wording from the default Portal billing workspace
- improves Step 06 `8.6` evidence by aligning the settlement-summary and explicit sandbox surface with a product-facing billing vocabulary
- does not close Step 06 globally because explicit simulation tooling, manual settlement bridge behavior, and compatibility fallback posture still remain in runtime
- does not require `docs/架构/*` writeback because this slice changes Portal presentation copy, i18n coverage, and source-contract proof only; it does not change backend contracts, route authority, or ownership boundaries
- `97` is satisfied through truthful no-writeback classification for this iteration

## Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-workspace.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## Remaining Follow-Up

1. Continue reducing remaining operator-oriented copy from Portal billing without inventing frontend-only payment behavior.
2. Decide later whether the explicit payment-event sandbox should stay visible in the current workbench layout or move behind an even narrower operator-only boundary.
3. Continue Step 06 Portal commercialization closure until the full billing journey reads as a tenant product surface end to end.
