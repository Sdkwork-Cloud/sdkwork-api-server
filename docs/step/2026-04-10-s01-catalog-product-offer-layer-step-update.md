# 2026-04-10 S01 Catalog Product/Offer Layer Step Update

## Scope

- Step: `S01 API product and pricing model convergence`
- Loop target: add a canonical `product / offer` read layer to portal commerce catalog without breaking existing `plans / packs / recharge_options / custom_recharge_policy / coupons`
- Boundaries: app-commerce catalog assembly, portal commerce catalog contract, portal OpenAPI, portal shared TS types

## Changes

- Catalog:
  - added additive `products`
  - added additive `offers`
  - kept existing `plans`
  - kept existing `packs`
  - kept existing `recharge_options`
  - kept existing `custom_recharge_policy`
  - kept existing `coupons`
- Added canonical read DTOs:
  - `PortalApiProduct`
  - `PortalProductOffer`
- Derived canonical catalog truth from current commerce sources:
  - subscription plans -> product + offer
  - recharge packs -> product + offer
  - custom recharge policy -> product + offer
- OpenAPI:
  - published `/portal/commerce/catalog`
  - published `PortalCommerceCatalog`
  - published `PortalApiProduct`
  - published `PortalProductOffer`
- Portal TS:
  - added `PortalApiProduct`
  - added `PortalProductOffer`
  - extended `PortalCommerceCatalog` with `products` and `offers`

## Verification

- `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_catalog_exposes_plans_packs_and_active_coupons -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_catalog_exposes_server_managed_recharge_options_and_custom_policy -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_quote_prices_recharge_and_coupon_redemption -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test openapi_route openapi_routes_expose_portal_api_inventory_with_schema_components -- --nocapture`
- `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure quote_preview -- --nocapture`
- `node --input-type=module -` with direct assertions over `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`

## Result

- Portal catalog regression: passed
- Portal quote regression: passed
- Portal OpenAPI regression: passed
- App-commerce quote regression: passed
- Portal TS catalog contract assertions: passed
- Node built-in test runner remained blocked by environment `spawn EPERM`; TS contract closure used equivalent direct assertions instead

## Next Gate

- Continue `S01` by converging domain/app canonical `ApiProduct / ProductOffer / CatalogPublication / Pricing` truth
- Keep `S02 || S03` blocked from deep parallel rollout until shared product and pricing owners are stabilized
