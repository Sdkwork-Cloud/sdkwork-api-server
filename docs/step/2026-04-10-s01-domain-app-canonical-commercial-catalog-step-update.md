# 2026-04-10 S01 Domain/App Canonical Commercial Catalog Step Update

## Scope

- Step: `S01 API product and pricing model convergence`
- Loop target: replace portal-local catalog assembly as the primary truth with a shared domain/app canonical commercial catalog layer
- Boundaries: `sdkwork-api-domain-catalog`, `sdkwork-api-app-catalog`, `sdkwork-api-app-commerce`, verification-enabling sqlite catalog support

## Changes

- Domain catalog:
  - added `ApiProductKind`
  - added `QuoteKind`
  - added `CatalogPublicationKind`
  - added `CatalogPublicationStatus`
  - added `ApiProduct`
  - added `ProductOffer`
  - added `CatalogPublication`
  - added `CommercialCatalog`
- App catalog:
  - added `CommercialCatalogSeedProduct`
  - added canonical ID builders for product, offer, and publication
  - added `build_canonical_commercial_catalog(...)`
  - re-exported canonical commercial catalog types for service-layer use
- App commerce:
  - replaced portal-local `products / offers` assembly as the source of truth
  - now derives portal catalog `products / offers` from the shared canonical catalog builder
  - preserved outward portal DTO and legacy catalog fields
- Verification enablement:
  - imported `ProviderAccountRecord` into sqlite storage
  - renamed catalog string-list codecs to avoid helper-name ambiguity
  - switched provider-account listing to explicit `SqliteRow` decoding so fresh rebuild verification can complete
  - removed the obsolete provider-account tuple decoder path after the sqlite-row cutover

## Verification

- `cargo test -p sdkwork-api-domain-catalog --test commercial_catalog -- --nocapture`
- `cargo test -p sdkwork-api-app-catalog --test commercial_catalog -- --nocapture`
- `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure quote_preview -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_catalog_exposes_plans_packs_and_active_coupons -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_catalog_exposes_server_managed_recharge_options_and_custom_policy -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test openapi_route openapi_routes_expose_portal_api_inventory_with_schema_components -- --nocapture`

## Result

- shared canonical commercial catalog truth now exists below portal compatibility DTOs
- portal catalog `products / offers` now reuse domain/app canonical semantics instead of ad hoc assembly
- fresh rebuild verification recovered without widening storage-schema scope

## Next Gate

- continue `S01` by converging canonical `PricingPlan / PricingRate / Publication` ownership onto the same shared product/offer model
- keep `S02 || S03` from deep parallel rollout until shared pricing truth is attached to the canonical catalog
