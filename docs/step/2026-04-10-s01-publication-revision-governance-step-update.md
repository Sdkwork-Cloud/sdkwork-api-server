# 2026-04-10 S01 Commercial Publication-Revision Governance Step Update

## Scope

- Step: `S01 API product and pricing model convergence`
- Loop target: turn `CatalogPublication` from status-only projection into explicit publication revision evidence
- Boundaries: `sdkwork-api-domain-catalog`, `sdkwork-api-app-catalog`, `sdkwork-api-app-commerce`, `sdkwork-api-interface-portal`

## Changes

- Canonical publication model:
  - added `publication_revision_id`
  - added `publication_version`
  - added `publication_source_kind`
  - added `publication_effective_from_ms`
- Catalog runtime:
  - publication revision evidence is now derived explicitly from governed pricing-plan truth or fallback catalog seed truth
  - canonical revision id now follows `publication_revision:<publication_kind>:offer:<product_kind>:<target_id>:v<version>`
- Commerce runtime:
  - threaded publication revision evidence through catalog binding, quote, order snapshot, and settlement-side quote reconstruction
  - preserved existing `pricing_plan_*` fields; publication semantics are additive, not replacing pricing identity
- Portal contract:
  - exposed publication revision evidence on catalog offers, order views, and manual OpenAPI schema

## Verification

- RED:
  - `cargo test -p sdkwork-api-app-catalog --test commercial_catalog canonical_catalog_prefers_active_pricing_governance_over_newer_planned_version -- --nocapture`
  - `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure preview_quote_prefers_active_pricing_governance_from_account_kernel -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_catalog_prefers_active_pricing_governance_from_account_kernel -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test openapi_route openapi_routes_expose_portal_api_inventory_with_schema_components -- --nocapture`
- GREEN:
  - `cargo test -p sdkwork-api-domain-catalog --test commercial_catalog -- --nocapture`
  - `cargo test -p sdkwork-api-app-catalog --test commercial_catalog -- --nocapture`
  - `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test openapi_route -- --nocapture`

## Result

- `publication` now carries explicit revision evidence instead of only `id / kind / status`
- portal catalog, quote, order, and order snapshot can distinguish publication revision truth from pricing-plan identity
- future admin publication control can attach to an explicit outward revision contract without breaking current portal reads

## Architecture Backwrite

- checked architecture doc `166`
- no text change required this loop; implementation now matches the existing publication / schedule / publish governance direction more closely

## Next Gate

- continue `S01`
- next best slice: add admin-visible publication projection and channel-aware publication governance, then decide whether separate publication persistence is required
