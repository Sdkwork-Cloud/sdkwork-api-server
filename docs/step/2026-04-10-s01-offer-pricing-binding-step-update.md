# 2026-04-10 S01 Offer Pricing Binding Step Update

## Scope

- Step: `S01 API product and pricing model convergence`
- Loop target: attach canonical pricing binding to `ProductOffer` and stop using order `target_id` as a fake `pricing_plan_id`
- Boundaries: `sdkwork-api-domain-catalog`, `sdkwork-api-app-catalog`, `sdkwork-api-app-commerce`, portal contract tests, portal shared TS types

## Changes

- Domain catalog:
  - extended `ProductOffer` with additive pricing binding fields:
    - `pricing_plan_id`
    - `pricing_plan_version`
    - `pricing_rate_id`
    - `pricing_metric_code`
- App catalog:
  - added deterministic canonical pricing helpers:
    - pricing plan id
    - pricing rate id
    - pricing metric code
  - canonical commercial catalog builder now assigns pricing binding to every offer
- App commerce:
  - portal catalog `offers` now expose canonical pricing binding
  - added current quote-target pricing binding resolution from canonical offer truth
  - order creation now sources `pricing_plan_id / pricing_plan_version` from canonical offer binding instead of reusing subscription `target_id`
- Portal TS:
  - extended `PortalProductOffer` with additive pricing binding fields
- Test enablement:
  - added `serde_json` test dependency to `sdkwork-api-app-catalog` for runtime contract assertions

## Verification

- RED:
  - `cargo test -p sdkwork-api-app-catalog --test commercial_catalog -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_catalog_exposes_plans_packs_and_active_coupons -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_subscription_checkout_requires_settlement_before_membership_activation -- --nocapture`
- GREEN:
  - `cargo test -p sdkwork-api-domain-catalog --test commercial_catalog -- --nocapture`
  - `cargo test -p sdkwork-api-app-catalog --test commercial_catalog -- --nocapture`
  - `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure quote_preview -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_catalog_exposes_plans_packs_and_active_coupons -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_subscription_checkout_requires_settlement_before_membership_activation -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_order_detail_returns_canonical_order_view -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test openapi_route openapi_routes_expose_portal_api_inventory_with_schema_components -- --nocapture`
  - `node --input-type=module -` direct assertions over portal shared TS type pricing fields

## Result

- canonical `ProductOffer` now answers both `how to sell` and `which pricing plan/rate binds this offer`
- portal offer contract and order contract now share the same pricing-plan truth
- subscription orders no longer misuse product `target_id` as pricing-plan identity

## Next Gate

- continue `S01` by converging quote pricing evidence and `pricing_snapshot_json` onto the same canonical pricing binding
- keep `S02 || S03` parallel rollout conservative until quote/order/pricing publication evidence is fully aligned
