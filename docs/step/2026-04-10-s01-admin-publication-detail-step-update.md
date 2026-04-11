# 2026-04-10 S01 Commercial Admin Publication Detail Step Update

## Scope

- Step: `S01 API product and pricing model convergence`
- Loop target: upgrade admin publication read model from list projection to governed detail view
- Boundaries: `sdkwork-api-interface-admin`

## Changes

- Admin commerce route:
  - added `GET /admin/commerce/catalog-publications/{publication_id}`
  - returns `CommercialCatalogPublicationDetail`
- Detail model:
  - `projection`: canonical `product / offer / publication`
  - `governed_pricing_plan`: resolved backing `PricingPlanRecord` when publication truth comes from pricing governance
  - `governed_pricing_rates`: rates under the resolved pricing plan
- Governance resolution:
  - resolves pricing truth by canonical commercial `plan_code + publication_version`
  - keeps `catalog_seed` publications readable without inventing fake pricing governance
- Admin contract:
  - added detail route and schema to admin OpenAPI

## Verification

- RED:
  - `cargo test -p sdkwork-api-interface-admin admin_commerce_catalog_publication_detail_exposes_governed_pricing_context -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin openapi_routes_expose_admin_api_inventory_with_schema_components -- --nocapture`
- GREEN:
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test account_billing_routes -- --nocapture`

## Result

- admin can now inspect one publication together with its exact governed pricing plan and rates
- publication semantics stay explicit; pricing remains backing governance evidence rather than hidden operator knowledge
- next-step publication actions can now target a stable detail contract instead of jumping straight from list view to mutation

## Architecture Backwrite

- checked architecture doc `166`
- no text change required this loop; implementation now matches the publication lifecycle/control-plane direction more closely

## Next Gate

- continue `S01`
- next best slice: add semantic admin publication lifecycle actions or actionability hints, with explicit audit semantics instead of raw pricing-plan mental mapping
