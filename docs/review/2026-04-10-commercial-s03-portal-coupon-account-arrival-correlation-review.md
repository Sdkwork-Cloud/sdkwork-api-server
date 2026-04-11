# 2026-04-10 Commercial S03 Portal Coupon Account Arrival Correlation Review

## Scope

- Architecture reference: architecture doc `166`
- Step reference: step doc `105`
- Loop focus: close grant-style coupon reward-history account-arrival evidence on portal

## Findings

### P1 - reward-history stopped at redemption status, not到账 evidence

- `reward-history` exposed coupon redemption and rollback facts, but not whether grant-style redemption actually created account benefit lots
- impact:
  - users and support had to cross-read billing/account pages to confirm arrival
  - coupon-first portal semantics stayed incomplete for `account_entitlement`

### P1 - outward contract hid the real correlation chain already present in runtime

- runtime already persisted:
  - `CouponRedemptionRecord.order_id`
  - `AccountBenefitLotRecord.scope_json.order_id`
  - `AccountBenefitLotRecord.source_type/source_id`
- impact:
  - portal omitted proven evidence and forced manual debugging

## Fix Closure

- added additive `account_arrival` on `PortalMarketingRewardHistoryItem`
- correlated current-subject reward-history redemptions to current workspace commercial-account lots by `order_id`
- restricted linked lot evidence to `source_type = order`
- synchronized Rust routes, manual OpenAPI, shared TS types, and portal reward-history UI copy

## Verification

- RED:
  - route test failed on missing `account_arrival.order_id`
  - OpenAPI test failed on missing `PortalCouponAccountArrivalSummary`
  - TS contract test failed on missing `PortalCouponAccountArrival*` interfaces
  - portal page test failed on missing arrival-specific UI copy
- GREEN:
  - `cargo test -p sdkwork-api-interface-portal --test marketing_coupon_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test openapi_route -- --nocapture`
  - `node ./apps/sdkwork-router-portal/tests/portal-marketing-api-surface.test.mjs`
  - `node ./apps/sdkwork-router-portal/tests/portal-marketing-coupon-flow.test.mjs`
  - `node ./apps/sdkwork-router-portal/tests/portal-redeem-growth-polish.test.mjs`

## Residual Risks

- wider public API still does not expose the same reward-history account-arrival contract
- correlation currently scans current-account lots in memory; indexed/query-optimized projection can stay deferred until real volume requires it

## Exit

- Step result: `conditional-go`
- Reason:
  - portal grant-style coupon到账 evidence is now explicit and evidence-based
  - `S03` outward convergence still needs public API closure
