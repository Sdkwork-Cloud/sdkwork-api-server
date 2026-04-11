# 2026-04-10 Commercial S06 Public Market/Coupon/Commercial API Convergence Review

## Scope

- Architecture reference: architecture doc `166`
- Step reference: step doc `108`
- Loop focus: close public gateway runtime and documentation drift for `market / marketing / commercial`

## Findings

### P1 - architecture route matrix existed, but public gateway runtime still returned 404

- `166 / 8.3` had already frozen the public route set:
  - `/market/products`
  - `/market/offers`
  - `/market/quotes`
  - `/marketing/coupons/*`
  - `/commercial/account*`
- impact:
  - public API productization stopped at architecture intent
  - coupon-first convergence was portal-biased and not available from the outward gateway surface

### P1 - runtime and `/openapi.json` diverged after route closure

- once runtime routes were added, `/openapi.json` still omitted all 9 public routes and their schemas
- impact:
  - external integration had no contract truth
  - client generation and partner onboarding would rely on reverse-engineering live responses

## Fix Closure

- added the 9 public runtime routes to `sdkwork-api-interface-http`
- kept outward coupon semantics explicit with `template / campaign / applicability / effect`
- exposed `scope_order_id` on commercial benefit lots for outward order-to-arrival evidence
- added gateway OpenAPI tags `market / marketing / commercial` and published the same runtime contract into `/openapi.json`
- added route-level regressions for both runtime behavior and OpenAPI inventory

## Verification

- RED:
  - `market_coupon_commercial_routes` failed on missing runtime routes
  - `openapi_route` failed on missing `/market/products` and missing public schemas
- GREEN:
  - `cargo test -p sdkwork-api-interface-http --test market_coupon_commercial_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-http --test openapi_route -- --nocapture`
  - `cargo test -p sdkwork-api-interface-http --test canonical_account_admission -- --nocapture`
  - `cargo test -p sdkwork-api-interface-http --test market_coupon_commercial_routes --test openapi_route --test canonical_account_admission -- --nocapture`

## Residual Risks

- `GET /commercial/account/benefit-lots` still uses store-wide lot listing plus in-memory account filtering; high-volume account histories still need account-scoped query/index convergence
- this loop verified runtime and OpenAPI only; downstream SDK/client regeneration from the public gateway schema is not yet revalidated

## Exit

- Step result: `conditional-go`
- Reason:
  - public runtime and contract drift is closed
  - scale and downstream consumer follow-ups remain
