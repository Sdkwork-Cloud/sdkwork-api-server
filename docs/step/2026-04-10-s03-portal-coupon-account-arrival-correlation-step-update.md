# 2026-04-10 S03 Portal Coupon Account Arrival Correlation Step Update

## Scope

- Step: `S03 Coupon-First marketing model convergence`
- Loop target: close portal reward-history account-arrival evidence for grant-style coupons
- Boundaries:
  - `sdkwork-api-interface-portal`
  - portal manual OpenAPI
  - portal shared TS types
  - portal credits workspace

## Changes

- Contract:
  - added additive `account_arrival` to `/portal/marketing/reward-history`
  - exposed:
    - `order_id`
    - `account_id`
    - `benefit_lot_count`
    - `credited_quantity`
    - `benefit_lots`
- Correlation:
  - linked reward-history redemption evidence through:
    - `CouponRedemptionRecord.order_id`
    - current workspace commercial account
    - `AccountBenefitLotRecord.scope_json.order_id`
    - `AccountBenefitLotRecord.source_type = order`
- UI:
  - upgraded reward-history rows to render:
    - `Arrived to account`
    - `No linked account lot evidence yet`
    - `No account arrival for checkout discount`
- Compatibility:
  - kept the slice additive; existing wallet/history semantics remain valid

## Verification

- RED:
  - reward-history route test failed because grant-style redemptions exposed no linked account-arrival evidence
  - OpenAPI test failed because `PortalMarketingRewardHistoryItem` lacked `account_arrival`
  - portal TS surface test failed because shared types lacked account-arrival interfaces
  - portal coupon flow page test failed because reward-history UI had no arrival semantics
- GREEN:
  - `cargo test -p sdkwork-api-interface-portal --test marketing_coupon_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test openapi_route -- --nocapture`
  - `node ./apps/sdkwork-router-portal/tests/portal-marketing-api-surface.test.mjs`
  - `node ./apps/sdkwork-router-portal/tests/portal-marketing-coupon-flow.test.mjs`
  - `node ./apps/sdkwork-router-portal/tests/portal-redeem-growth-polish.test.mjs`

## Result

- portal reward-history now distinguishes redeemed discount coupons from redeemed account-entitlement coupons that have real linked account lots
- grant-style coupon到账 evidence is now visible from the same coupon-first reward-history contract instead of requiring a separate billing page join

## Exit

- Step result: `conditional-go`
- Reason:
  - portal-side reward-history account-arrival correlation is closed
  - wider public API convergence on the same outward coupon contract remains open
