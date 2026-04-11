# 2026-04-10 S01 Catalog Publication Owner Chain Step Update

## Scope

- Step: `S01 API product and pricing model convergence`
- Loop target: converge `ApiProduct / ProductOffer / CatalogPublication` owner chain into portal catalog offers, quote payloads, and order snapshot evidence
- Boundaries: `sdkwork-api-app-commerce`, portal contract, portal shared TS types, quote/order regression tests

## Changes

- App commerce:
  - extended canonical quote binding from pricing-only to full catalog owner chain:
    - `product_id`
    - `offer_id`
    - `publication_id`
    - `publication_kind`
    - `publication_status`
    - existing pricing binding fields
  - quote construction now resolves owner chain from canonical commercial catalog
  - settlement-side quote rehydration now restores owner chain from stored snapshot first, then compatibility-falls back to live canonical catalog when old snapshots lack those fields
  - `pricing_snapshot_json` now freezes explicit `catalog_binding` evidence beside `pricing_binding`
- Portal catalog:
  - `offers` now expose publication evidence, not only product and pricing references
- Portal quote:
  - recharge-pack and custom-recharge quote flows now expose canonical `product_id / offer_id / publication_*`
- Portal TS:
  - extended `PortalProductOffer` and `PortalCommerceQuote` with additive publication owner-chain fields

## Verification

- RED:
  - `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure quote_preview -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_quote_prices_recharge_and_coupon_redemption -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_catalog_exposes_plans_packs_and_active_coupons -- --nocapture`
- GREEN:
  - `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure quote_preview -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_catalog_exposes_plans_packs_and_active_coupons -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_quote_prices_recharge_and_coupon_redemption -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_quote_and_order_support_custom_recharge_from_server_policy -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_subscription_checkout_requires_settlement_before_membership_activation -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test openapi_route openapi_routes_expose_portal_api_inventory_with_schema_components -- --nocapture`
  - `node --input-type=module -` direct assertions over portal shared TS owner-chain fields

## Result

- quote payload now points at canonical `ApiProduct / ProductOffer / CatalogPublication / Pricing`
- portal catalog offer and portal quote now share one owner chain instead of parallel local inference
- order snapshot now preserves catalog publication evidence needed for later order-side canonicalization

## Architecture Backwrite

- checked `docs/架构/166-API产品-商业账户-coupon-first营销统一架构-2026-04-10.md`
- no text change required this loop; implementation now catches up to the existing `Product / Offer / Publication / Quote` layering target

## Next Gate

- continue `S01` by exposing canonical owner chain on outward order views and tightening publication governance ownership
- keep `S02 || S03` expansion conservative until order read contract also stops relying on implicit snapshot-only owner evidence
