# 2026-04-10 S06 Public API Reference Consumer Closure Step Update

## Scope

- Step: `S06 Portal and Public API productization closure`
- Loop target: close the remaining public consumer documentation gap for `market / marketing / commercial`
- Boundaries:
  - `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-reference`
  - `docs/api-reference/gateway-api.md`
  - `apps/sdkwork-router-portal/tests/portal-api-reference-center.test.mjs`

## Changes

- API reference center:
  - added explicit tag-label mapping for:
    - `market`
    - `marketing`
    - `commercial`
  - upgraded gateway reference focus copy to include market, coupon, and commercial account workflows
- Gateway API reference doc:
  - documented public route families:
    - `market`
    - `marketing`
    - `commercial`
  - documented public benefit-lot traversal fields:
    - `after_lot_id`
    - `next_after_lot_id`
    - `scope_order_id`

## Verification

- RED:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-api-reference-center.test.mjs`
    - failed because `docs/api-reference/gateway-api.md` still lacked the public `market / marketing / commercial` route-family contract
- GREEN:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-api-reference-center.test.mjs`

## Result

- the public developer-facing reference center now describes the same coupon-first market and commercial surface that runtime and OpenAPI already expose

## Exit

- Step result: `conditional-go`
- Reason:
  - public consumer-facing route-family documentation is now closed
  - downstream SDK / client regeneration against the updated gateway schema is still pending
