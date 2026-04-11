# 2026-04-10 S01 Quote Pricing Evidence Step Update

## Scope

- Step: `S01 API product and pricing model convergence`
- Loop target: converge `PortalCommerceQuote` and `pricing_snapshot_json` onto the same canonical offer pricing binding
- Boundaries: `sdkwork-api-app-commerce`, portal quote contract, portal shared TS types, quote/order regression tests

## Changes

- App commerce:
  - extended `PortalCommerceQuote` with additive canonical pricing binding fields:
    - `pricing_plan_id`
    - `pricing_plan_version`
    - `pricing_rate_id`
    - `pricing_metric_code`
  - `build_priced_quote(...)` now resolves pricing binding from canonical offer truth and writes it into quote payloads
  - `submit_portal_commerce_order(...)` now persists `pricing_plan_id / pricing_plan_version` from quote truth instead of recomputing separately
  - `pricing_snapshot_json` now freezes explicit `pricing_binding` evidence alongside the serialized quote
  - settlement-side quote rehydration now reuses stored pricing evidence from order snapshot/order record
- Portal contract:
  - quote API now exposes canonical pricing binding for recharge-pack and custom-recharge quote flows
- Portal TS:
  - extended `PortalCommerceQuote` with the same additive pricing binding fields

## Verification

- RED:
  - `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure quote_preview -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_quote_prices_recharge_and_coupon_redemption -- --nocapture`
- GREEN:
  - `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure quote_preview -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_quote_prices_recharge_and_coupon_redemption -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_quote_and_order_support_custom_recharge_from_server_policy -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_subscription_checkout_requires_settlement_before_membership_activation -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_order_detail_returns_canonical_order_view -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test openapi_route openapi_routes_expose_portal_api_inventory_with_schema_components -- --nocapture`
  - `node --input-type=module -` direct assertions over portal shared TS quote pricing fields

## Result

- quote payload now exposes the same canonical pricing identity already carried by catalog offer truth
- order pricing evidence now freezes quote-time pricing binding instead of leaving rate/metric semantics implicit
- custom recharge quote flow now shares the same canonical pricing-binding contract as standard offer-backed flows

## Architecture Backwrite

- checked `docs/架构/166-API产品-商业账户-coupon-first营销统一架构-2026-04-10.md`
- no architecture text delta was required this loop; implementation now catches up to existing `Quote / Pricing / Order evidence` intent

## Next Gate

- continue `S01` by converging quote/order evidence with explicit catalog publication evidence
- keep `S02 || S03` rollout conservative until `Quote -> Order Snapshot -> Publication` owner chain is explicit
