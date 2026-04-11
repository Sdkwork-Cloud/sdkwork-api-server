# 2026-04-10 S01 Commercial Plan-Code Governance Step Update

## Scope

- Step: `S01 API product and pricing model convergence`
- Loop target: turn commercial pricing `plan_code` from loose naming into shared canonical governance
- Boundaries: `sdkwork-api-app-catalog`, `sdkwork-api-interface-admin`, commercial pricing CRUD tests

## Changes

- Shared helper:
  - added `normalize_commercial_pricing_plan_code(...)`
  - canonical commercial codes now normalize to `<product_kind>:<target_id>`
  - accepted product-kind variants now converge to canonical snake-case semantics
  - malformed commercial codes with empty `target_id` are rejected explicitly
- Catalog runtime:
  - pricing-governance lookup now uses the shared helper instead of raw string equality
  - existing commercial plan rows like `Subscription-Plan : growth` now still align to canonical catalog truth
- Admin pricing:
  - create/update pricing-plan handlers now normalize canonical commercial codes before persistence
  - generic non-commercial plan codes remain compatibility-safe and unchanged

## Verification

- RED:
  - `cargo test -p sdkwork-api-app-catalog --test commercial_catalog normalizes_canonical_commercial_pricing_plan_code_variants -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test account_billing_routes admin_billing_pricing_management_routes_normalize_commercial_plan_codes_on_create_and_update -- --nocapture`
- GREEN:
  - `cargo test -p sdkwork-api-app-catalog --test commercial_catalog -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test account_billing_routes admin_billing_pricing_management_routes_normalize_commercial_plan_codes_on_create_and_update -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test account_billing_routes admin_billing_pricing_management_routes_reject_commercial_plan_codes_without_target_id -- --nocapture`

## Result

- canonical commercial pricing alignment no longer depends on operators typing the exact string shape everywhere
- admin pricing CRUD now writes one canonical commercial `plan_code` truth for product-bound pricing
- catalog governance can consume older commercial code variants without requiring storage migration

## Architecture Backwrite

- checked architecture doc `166`
- no text change required this loop; implementation now matches the existing “coupon/market/admin share explicit commercial truth” direction more closely

## Next Gate

- continue `S01`
- next best slice: promote `CatalogPublication` from derived outward evidence into admin-governed first-class publication revision truth
