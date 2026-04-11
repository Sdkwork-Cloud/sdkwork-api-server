# 2026-04-10 Commercial S03 Portal Coupon Semantic Convergence Review

## Scope

- Architecture reference: architecture doc `166`
- Step reference: step doc `105`
- Loop focus: make portal coupon surfaces speak explicit coupon-first business semantics

## Findings

### P1 - portal wallet exposed codes but not coupon meaning

- `my-coupons` mainly returned `code + latest reservation/redemption`
- impact:
  - users could not tell what a coupon was, what campaign it belonged to, or what effect it carried
  - grant-style coupons and discount-style coupons looked semantically identical

### P1 - portal did not expose applicability truth

- portal coupon surfaces did not carry explicit applicable target scope
- impact:
  - coupon usage remained guess-based
  - portal stayed behind the architecture target that requires “what products/targets this coupon applies to”

### P2 - ownership truth was implicit

- visibility came from claim/reservation/redemption, but outward contract did not explain current-subject ownership state
- impact:
  - portal had “my coupons” naming without explicit ownership evidence

## Fix Closure

- added semantic coupon read-model fields on portal wallet/history:
  - `template`
  - `campaign`
  - `applicability`
  - `effect`
  - `ownership`
- split coupon effect outwardly into:
  - `checkout_discount`
  - `account_entitlement`
- surfaced `all_target_kinds_eligible + target_kinds`
- surfaced `owned_by_current_subject + claimed_to_current_subject`
- synchronized Rust routes, manual OpenAPI, shared TS types, and portal credits UI

## Verification

- RED:
  - portal marketing route test failed on missing `template / effect`
  - OpenAPI test failed on missing semantic coupon schemas
  - portal marketing TS surface test failed on missing semantic coupon interfaces
- GREEN:
  - `cargo test -p sdkwork-api-interface-portal --test marketing_coupon_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test openapi_route -- --nocapture`
  - `node ./apps/sdkwork-router-portal/tests/portal-marketing-api-surface.test.mjs`
  - `node ./apps/sdkwork-router-portal/tests/portal-marketing-coupon-flow.test.mjs`
  - `node ./apps/sdkwork-router-portal/tests/portal-redeem-growth-polish.test.mjs`

## Residual Risks

- grant-style coupon到账权益 still lacks per-redemption account-lot correlation
- portal/public API convergence is still not fully closed outside the portal wallet/history slice
- compatibility flows still coexist with the preferred semantic outward contract

## Exit

- Step result: `conditional-go`
- Reason:
  - portal coupon semantics now meet the next outward professionalism bar
  - `S03` still needs account-arrival evidence and public API convergence
