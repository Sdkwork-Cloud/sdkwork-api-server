# 2026-04-10 Commercial S01 Domain App Canonical Commercial Catalog Review

## Scope

- Architecture reference: `docs/架构/166-API产品-商业账户-coupon-first营销统一架构-2026-04-10.md`
- Step reference: `docs/step/103-S01-API产品与定价主模型收敛-2026-04-10.md`
- Loop focus: establish shared domain/app `ApiProduct / ProductOffer / CatalogPublication` truth and remove portal-local catalog assembly as the primary semantic owner

## Findings

### P0 - canonical product/offer semantics existed only as a portal compatibility projection

- the previous loop exposed `products / offers` outward, but their truth was still assembled inside portal-facing commerce code
- impact:
  - `ApiProduct / ProductOffer / CatalogPublication` semantics were not reusable by admin, market, pricing, or future account-entitlement flows
  - coupon-first commercialization would keep growing around a compatibility projection instead of a shared model owner

### P1 - fresh rebuild verification was blocked by sqlite catalog/provider-account decode debt

- the new loop forced a clean rebuild and exposed compile blockers in sqlite catalog support
- impact:
  - S01 progress could not be verified from a fresh state
  - commercialization loops would keep depending on warm-cache success instead of reproducible verification

## Fix Closure

- added shared canonical commercial catalog records in `sdkwork-api-domain-catalog`
- added a reusable seed-to-canonical builder in `sdkwork-api-app-catalog`
- rewired `sdkwork-api-app-commerce` to map portal catalog `products / offers` from the canonical catalog
- kept outward portal DTOs and legacy catalog buckets stable
- repaired sqlite verification blockers by:
  - importing `ProviderAccountRecord`
  - separating catalog string-list codecs from other helpers
  - moving provider-account listing to explicit `SqliteRow` decoding
  - removing the obsolete provider-account tuple decoder compatibility path

## Verification

- `cargo test -p sdkwork-api-domain-catalog --test commercial_catalog -- --nocapture`
- `cargo test -p sdkwork-api-app-catalog --test commercial_catalog -- --nocapture`
- `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure quote_preview -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_catalog_exposes_plans_packs_and_active_coupons -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_catalog_exposes_server_managed_recharge_options_and_custom_policy -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test openapi_route openapi_routes_expose_portal_api_inventory_with_schema_components -- --nocapture`

## Residual Risks

- canonical pricing ownership is still separate from the new product/offer/publication catalog
- portal compatibility fields remain active, so deeper consumer cutover is still pending

## Exit

- Step result: `conditional-go`
- Reason:
  - canonical product/offer/publication truth now exists at domain/app level
  - S01 still needs pricing/publication convergence before later-step parallelization is safe
