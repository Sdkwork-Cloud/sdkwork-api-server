# 2026-04-10 S03 Portal Coupon Semantic Convergence Step Update

## Scope

- Step: `S03 Coupon-First marketing model convergence`
- Loop target: close the next outward portal coupon semantic gap after admin revision governance
- Boundaries:
  - `sdkwork-api-interface-portal`
  - portal shared TS types
  - portal credits workspace

## Changes

- Contract:
  - upgraded `/portal/marketing/my-coupons` and `/portal/marketing/reward-history` from `code + status` payloads into coupon-semantic read models
  - added additive portal coupon views:
    - `template`
    - `campaign`
    - `applicability`
    - `effect`
    - `ownership`
- Semantics:
  - made coupon effect explicit as:
    - `checkout_discount`
    - `account_entitlement`
  - surfaced applicable target scope through `all_target_kinds_eligible + target_kinds`
  - surfaced current-subject ownership truth through `owned_by_current_subject + claimed_to_current_subject`
- UI:
  - upgraded portal credits wallet/history cells so users can directly read:
    - what the coupon is
    - what effect it has
    - what targets it applies to
- Support:
  - restored sqlite compile continuity by importing missing `PricingPlanOwnershipScope` in `account_support.rs`

## Verification

- RED:
  - portal route test failed because `my-coupons / reward-history` did not expose semantic coupon fields
  - OpenAPI test failed because portal schema lacked semantic coupon components
  - portal marketing API surface test failed because TS types lacked semantic coupon contracts
- GREEN:
  - `cargo test -p sdkwork-api-interface-portal --test marketing_coupon_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test openapi_route -- --nocapture`
  - `node ./apps/sdkwork-router-portal/tests/portal-marketing-api-surface.test.mjs`
  - `node ./apps/sdkwork-router-portal/tests/portal-marketing-coupon-flow.test.mjs`
  - `node ./apps/sdkwork-router-portal/tests/portal-redeem-growth-polish.test.mjs`

## Result

- portal coupon surfaces now explain coupon identity, campaign context, target applicability, effect mode, and current-subject ownership
- grant-style coupons and discount-style coupons are no longer flattened into the same opaque wallet row semantics
- `S03` outward convergence moved from admin-only governance to portal coupon semantic truth

## Exit

- Step result: `conditional-go`
- Reason:
  - portal coupon semantics are materially converged
  - per-redemption account benefit-lot arrival correlation and wider public API convergence remain open
