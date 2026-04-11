# 2026-04-10 Commercial S01 Outward Order Owner Chain Review

## Scope

- Architecture reference: `docs/架构/166-API产品-商业账户-coupon-first营销统一架构-2026-04-10.md`
- Step reference: `docs/step/103-S01-API产品与定价主模型收敛-2026-04-10.md`
- Loop focus: push canonical owner-chain semantics from catalog/quote/snapshot into outward portal order reads

## Findings

### P0 - outward order reads still hid canonical product and publication identity

- portal order detail and order-center still exposed only raw order storage fields plus `product_kind / transaction_kind`
- impact:
  - callers still needed implicit knowledge of `pricing_snapshot_json`
  - `Product / Offer / Publication / Quote / Order` layering was still incomplete at the read contract boundary

### P0 - portal order TS contract lagged runtime truth

- `PortalCommerceOrder` did not yet declare even the already-live `pricing_plan_id / pricing_plan_version`
- impact:
  - portal shared type layer understated the actual contract
  - later owner-chain rollout would become harder to reason about and verify

### P1 - interface layer risked duplicating catalog-binding reconstruction logic

- if portal interface reimplemented snapshot parsing locally, owner-chain truth would fork again
- impact:
  - future order/settlement/catalog changes would need duplicate maintenance

## Fix Closure

- promoted `PortalCommerceCatalogBinding` as reusable app-layer projection truth
- exposed app-layer order-binding helper for interface consumption
- extended portal order outward projection with canonical owner-chain and rate/metric evidence
- updated OpenAPI and portal shared TS types to match runtime payload shape
- verified order detail, order-center, openapi, quote regression, and app-commerce snapshot regression together

## Verification

- RED:
  - portal order-detail regression failed because outward order view had no `product_id`
  - portal order-center regression failed for the same reason on aggregate entries
  - openapi regression failed because `PortalCommerceOrder` schema lacked owner-chain fields
- GREEN:
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_order_detail_returns_canonical_order_view -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_order_center_aggregates_order_payment_and_checkout_views -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test openapi_route openapi_routes_expose_portal_api_inventory_with_schema_components -- --nocapture`
  - `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure quote_preview -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_quote_prices_recharge_and_coupon_redemption -- --nocapture`
  - `node --input-type=module -` direct portal TS contract assertions

## Residual Risks

- `CatalogPublication` ownership is still generated from canonical builder defaults, not yet linked to admin-controlled publication lifecycle
- pre-upgrade orders can only expose owner-chain fields if snapshot or canonical fallback can reconstruct them
- S01 still lacks explicit governance proof that publication identity is stable across future pricing/publication revisions

## Exit

- Step result: `conditional-go`
- Reason:
  - outward `Order` read contract now participates in the canonical owner chain
  - publication-governance ownership is still the main remaining S01 gap
