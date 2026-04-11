# 2026-04-10 Commercial S01 Catalog Product Offer Layer Review

## Scope

- Architecture reference: `docs/架构/166-API产品-商业账户-coupon-first营销统一架构-2026-04-10.md`
- Step reference: `docs/step/103-S01-API产品与定价主模型收敛-2026-04-10.md`
- Loop focus: add a canonical product/offer commerce catalog read layer without breaking current portal consumers

## Findings

### P0 - portal commerce catalog exposed only legacy sales views, not canonical product/offer truth

- catalog output was limited to `plans`, `packs`, `recharge_options`, `custom_recharge_policy`, and `coupons`
- impact:
  - `ApiProduct / ProductOffer` convergence had no outward landing zone
  - future market API, account entitlements, and publication modeling would need duplicate projections
  - coupon-first commercialization would keep coupling business semantics to legacy catalog buckets

### P1 - manual OpenAPI did not publish the catalog contract as an explicit commerce capability

- `/portal/commerce/catalog` and its schemas were missing from the manual portal OpenAPI document
- impact:
  - generated or human consumers could not reliably discover the catalog contract
  - contract drift risk was higher than necessary during commercialization refactors

## Fix Closure

- added additive `products` and `offers` to `PortalCommerceCatalog`
- added canonical read DTOs:
  - `PortalApiProduct`
  - `PortalProductOffer`
- projected product/offer truth from current plan, pack, and custom recharge sources
- published the catalog path and schemas in manual portal OpenAPI
- extended portal TS shared types and contract tests without removing old fields

## Verification

- `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_catalog_exposes_plans_packs_and_active_coupons -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_catalog_exposes_server_managed_recharge_options_and_custom_policy -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_quote_prices_recharge_and_coupon_redemption -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test openapi_route openapi_routes_expose_portal_api_inventory_with_schema_components -- --nocapture`
- `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure quote_preview -- --nocapture`
- `node --input-type=module -` direct semantic assertions for portal shared types

## Residual Risks

- this loop adds an outward canonical catalog layer, but domain-level `ApiProduct / ProductOffer / CatalogPublication / PricingPlan` convergence is still incomplete
- portal consumers still have compatibility access to legacy fields, so deeper UI and service cutover remains pending
- Node built-in test runner remains environment-blocked by `spawn EPERM`, so TS proof used equivalent direct assertions instead

## Exit

- Step result: `conditional-go`
- Reason:
  - outward catalog truth now has an explicit product/offer layer
  - S01 still needs deeper domain/app canonical model extraction before parallel expansion into later steps
