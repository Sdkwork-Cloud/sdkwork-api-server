# 2026-04-10 S06 Public Market/Coupon/Commercial API Convergence Step Update

## Scope

- Step: `S06 Portal and Public API productization closure`
- Loop target: close public gateway runtime and OpenAPI convergence for coupon-first `market / marketing / commercial` routes
- Boundaries:
  - `sdkwork-api-interface-http` runtime routes
  - gateway OpenAPI inventory
  - public route verification

## Changes

- Runtime:
  - added authenticated stateful public routes:
    - `GET /market/products`
    - `GET /market/offers`
    - `POST /market/quotes`
    - `POST /marketing/coupons/validate`
    - `POST /marketing/coupons/reserve`
    - `POST /marketing/coupons/confirm`
    - `POST /marketing/coupons/rollback`
    - `GET /commercial/account`
    - `GET /commercial/account/benefit-lots`
- Semantics:
  - kept coupon-first outward contract explicit with:
    - `template`
    - `campaign`
    - `applicability`
    - `effect`
  - made coupon effect explicit as:
    - `checkout_discount`
    - `account_entitlement`
  - exposed `scope_order_id` on commercial benefit lots so coupon redemption and account-arrival evidence can be correlated outwardly
- Contract:
  - added OpenAPI tags:
    - `market`
    - `marketing`
    - `commercial`
  - published the same 9 public routes into `/openapi.json`
  - added request/response schemas for public coupon/commercial payloads

## Verification

- RED:
  - `cargo test -p sdkwork-api-interface-http --test market_coupon_commercial_routes -- --nocapture`
    - previously failed on `404` because the public routes did not exist
  - `cargo test -p sdkwork-api-interface-http --test openapi_route -- --nocapture`
    - failed because `/openapi.json` omitted `/market/products` and the new public schemas
- GREEN:
  - `cargo test -p sdkwork-api-interface-http --test market_coupon_commercial_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-http --test openapi_route -- --nocapture`
  - `cargo test -p sdkwork-api-interface-http --test canonical_account_admission -- --nocapture`
  - `cargo test -p sdkwork-api-interface-http --test market_coupon_commercial_routes --test openapi_route --test canonical_account_admission -- --nocapture`

## Result

- public gateway runtime and `/openapi.json` now expose the same coupon-first market/commercial surface
- external consumers can discover API products, request quotes, execute coupon lifecycle actions, and inspect commercial account benefit-lot arrival evidence from one authenticated public surface

## Exit

- Step result: `conditional-go`
- Reason:
  - runtime and OpenAPI convergence is closed for the `166 / 8.3` route matrix
  - `/commercial/account/benefit-lots` still reads through store-wide listing plus account filtering and remains a scale follow-up
