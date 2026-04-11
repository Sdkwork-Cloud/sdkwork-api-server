# 2026-04-10 S01 Outward Order Owner Chain Step Update

## Scope

- Step: `S01 API product and pricing model convergence`
- Loop target: expose canonical owner chain on outward order read contract and remove portal order read dependence on hidden snapshot-only semantics
- Boundaries: `sdkwork-api-app-commerce`, `sdkwork-api-interface-portal`, portal shared TS types, order-view/openapi regression tests

## Changes

- App commerce:
  - promoted reusable `PortalCommerceCatalogBinding` as an app-layer projection contract
  - exposed `project_portal_commerce_order_catalog_binding(...)` so interface layer can read canonical `product / offer / publication / pricing` evidence without reimplementing snapshot parsing
- Portal interface:
  - extended `PortalCommerceOrderView` with additive fields:
    - `product_id`
    - `offer_id`
    - `publication_id`
    - `publication_kind`
    - `publication_status`
    - `pricing_rate_id`
    - `pricing_metric_code`
  - order detail and order-center responses now project canonical owner chain from snapshot/app-layer fallback instead of exposing only raw order storage truth
- Portal OpenAPI:
  - `PortalCommerceOrder` schema now documents owner-chain and pricing-binding fields already present in runtime contract
- Portal TS:
  - `PortalCommerceOrder` now includes canonical owner-chain fields plus previously missing `pricing_plan_id / pricing_plan_version`

## Verification

- RED:
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_order_detail_returns_canonical_order_view -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_order_center_aggregates_order_payment_and_checkout_views -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test openapi_route openapi_routes_expose_portal_api_inventory_with_schema_components -- --nocapture`
- GREEN:
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_order_detail_returns_canonical_order_view -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_order_center_aggregates_order_payment_and_checkout_views -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test openapi_route openapi_routes_expose_portal_api_inventory_with_schema_components -- --nocapture`
  - `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure quote_preview -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_quote_prices_recharge_and_coupon_redemption -- --nocapture`
  - `node --input-type=module -` direct assertions over portal shared TS order owner-chain fields

## Result

- portal outward order contract now exposes the same canonical owner chain already present in catalog, quote, and snapshot evidence
- portal order-center no longer hides canonical product/offer/publication identity behind internal snapshot-only data
- portal TS order contract now matches actual runtime payload more closely

## Architecture Backwrite

- checked `docs/架构/166-API产品-商业账户-coupon-first营销统一架构-2026-04-10.md`
- no text change required this loop; implementation now catches up to the existing `Product / Offer / Publication / Quote / Order` layering target

## Next Gate

- continue `S01` by tightening publication-governance ownership, so `CatalogPublication` identity stops being only builder-derived truth
- after that, reassess whether `S01` can move from `conditional-go` to `go`
