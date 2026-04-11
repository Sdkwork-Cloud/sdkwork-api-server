# 2026-04-10 Commercial S01 Coupon Semantics Split Review

## Scope

- Architecture reference: `docs/架构/166-API产品-商业账户-coupon-first营销统一架构-2026-04-10.md`
- Step reference: `docs/step/103-S01-API产品与定价主模型收敛-2026-04-10.md`
- Loop focus: remove mixed coupon/product semantics from outward commerce contracts while preserving compatibility

## Findings

### P0 - commerce outward contracts mixed product and coupon semantics in one field

- `target_kind` carried both product family semantics and coupon redemption semantics
- impact:
  - TS contracts were ambiguous
  - coupon-first marketing flows were harder to maintain
  - future account and API product modeling would inherit avoidable coupling

### P1 - direct order record exposure leaked storage semantics into portal contracts

- portal endpoints returned `PortalCommerceOrderRecord` directly
- impact:
  - additive semantic fields required either storage churn or duplicated frontend heuristics
  - coupon-first contract refinement was blocked at the interface layer

## Fix Closure

- added quote semantics:
  - `product_kind`
  - `quote_kind`
- added order semantics through interface-only wrapper:
  - `product_kind`
  - `transaction_kind`
- kept `target_kind` unchanged for compatibility
- split TS contract vocabulary into explicit commerce vs marketing aliases
- updated manual OpenAPI order schema to expose the additive fields

## Verification

- `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure quote_preview -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test openapi_route openapi_routes_expose_portal_api_inventory_with_schema_components -- --nocapture`
- `node --input-type=module -` semantic regex assertions for portal shared types

## Residual Risks

- portal billing UI still mostly reads `target_kind`; semantic display refinement can follow in a later loop
- Node built-in test runner remains environment-blocked by `spawn EPERM`, so this loop used equivalent direct assertions for the TS surface
- storage-layer order schema is intentionally unchanged in this loop; deeper S01 pricing/publication convergence is still pending

## Exit

- Step result: `conditional-go`
- Reason:
  - the semantic split required for coupon-first commercialization is now in place
  - broader S01 product/pricing publication convergence is still open
