# 2026-04-10 S01 Coupon Semantics Split Step Update

## Scope

- Step: `S01 API product and pricing model convergence`
- Loop target: separate product-purchase semantics from coupon-redemption semantics without breaking current `target_kind` contracts
- Boundaries: app-commerce quote surface, portal commerce response surface, portal OpenAPI, portal shared TS types

## Changes

- Quote:
  - kept `target_kind`
  - added additive `product_kind`
  - added additive `quote_kind`
- Order:
  - kept stored `CommerceOrderRecord` unchanged
  - added interface-layer `PortalCommerceOrderView`
  - added additive `product_kind`
  - added additive `transaction_kind`
- Portal TS:
  - added `PortalCommerceTargetKind`
  - added `PortalMarketingTargetKind`
  - added `ApiProductKind`
  - added `PortalQuoteKind`
  - added `CommercialTransactionKind`
  - kept `PortalCommerceQuoteKind` as compatibility alias

## Verification

- `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure quote_preview -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test openapi_route openapi_routes_expose_portal_api_inventory_with_schema_components -- --nocapture`
- `node --input-type=module -` with direct regex assertions over `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`

## Result

- Rust quote regression: passed
- Rust OpenAPI regression: passed
- Portal TS semantic contract assertions: passed
- Node built-in test runner remained blocked by environment `spawn EPERM`; product contract was verified through equivalent direct assertions instead

## Next Gate

- Continue `S01` by converging remaining API product / pricing publication semantics
- Then unlock `S02 || S03` in parallel once shared contract owners are fixed per file
