# 2026-04-10 S01 Commercial Admin Publication Projection Step Update

## Scope

- Step: `S01 API product and pricing model convergence`
- Loop target: expose canonical commercial publication truth to admin without creating a second publication system
- Boundaries: `sdkwork-api-app-commerce`, `sdkwork-api-interface-admin`

## Changes

- Shared runtime:
  - opened `current_canonical_commercial_catalog_for_store(...)` for cross-interface reuse instead of rebuilding catalog truth inside admin
- Admin commerce route:
  - added `GET /admin/commerce/catalog-publications`
  - returns `CommercialCatalogPublicationProjection { product, offer, publication }`
  - synchronizes due pricing lifecycle before projection, so admin sees the same governed publication truth as runtime commerce
- Admin contract:
  - added OpenAPI exposure for the new publication projection route
  - reused canonical `ApiProduct / ProductOffer / CatalogPublication` schemas instead of inventing duplicate admin-only fields

## Verification

- RED:
  - `cargo test -p sdkwork-api-interface-admin admin_commerce_catalog_publications_expose_canonical_product_offer_publication_chain -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin openapi_routes_expose_admin_api_inventory_with_schema_components -- --nocapture`
- GREEN:
  - `cargo test -p sdkwork-api-interface-admin admin_commerce_catalog_publications_expose_canonical_product_offer_publication_chain -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin openapi_routes_expose_admin_api_inventory_with_schema_components -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test account_billing_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route -- --nocapture`

## Result

- admin can now inspect canonical `product -> offer -> publication` evidence directly instead of inferring publication state from pricing-plan records
- publication revision evidence is no longer portal-only; operator-facing control-plane visibility now exists
- route semantics stay explicit: coupon remains marketing semantics, while publication stays commercial catalog semantics

## Architecture Backwrite

- checked architecture doc `166`
- no text change required this loop; implementation now matches the existing `ApiProduct / ProductOffer / CatalogPublication` governance direction more closely

## Next Gate

- continue `S01`
- next best slice: add admin publication detail/actions for `draft / schedule / publish / retire`, then evaluate channel-aware publication governance before separate publication persistence
