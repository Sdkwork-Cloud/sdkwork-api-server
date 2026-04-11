# 2026-04-10 S01 Pricing Governance Overlay Step Update

## Scope

- Step: `S01 API product and pricing model convergence`
- Loop target: stop hardcoding outward `pricing_plan_version / publication_status` when aligned pricing governance exists
- Boundaries: `sdkwork-api-app-catalog`, `sdkwork-api-app-commerce`, `sdkwork-api-storage-core`, sqlite/postgres admin stores, portal/app regression tests

## Changes

- Canonical catalog:
  - added deterministic commercial `plan_code` convention: `<product_kind>:<target_id>`
  - added `build_canonical_commercial_catalog_with_pricing_plans(...)`
  - catalog builder now overlays outward `pricing_plan_version` and `publication_status` from aligned pricing-plan governance
  - precedence is intentional:
    - `active/published` beats `planned/draft`
    - `planned/draft` beats `retired/archived`
    - fallback remains compatibility default `version=1` + `published`
- Storage abstraction:
  - added `AdminStore::account_kernel_store()` bridge so commerce can read pricing governance from the same store without widening all portal/admin state types
  - wired sqlite/postgres admin stores to expose their account-kernel surface
- Commerce runtime:
  - portal catalog now builds canonical commercial catalog from live pricing-plan governance when available
  - quote generation now uses store-backed catalog binding, so quote and order snapshot inherit governed `pricing_plan_version / publication_status`
  - order-read fallback remains compatibility-safe for pre-upgrade orders because snapshot/live default semantics are preserved

## Verification

- RED:
  - `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure preview_quote_prefers_active_pricing_governance_from_account_kernel -- --nocapture`
  - `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure preview_quote_marks_governed_planned_catalog_as_draft_when_no_active_plan_exists -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_catalog_prefers_active_pricing_governance_from_account_kernel -- --nocapture`
- GREEN:
  - `cargo test -p sdkwork-api-app-catalog --test commercial_catalog -- --nocapture`
  - `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure quote_preview -- --nocapture`
  - `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure submitted_order_freezes_quote_pricing_binding_in_snapshot -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_catalog_exposes_plans_packs_and_active_coupons -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_quote_prices_recharge_and_coupon_redemption -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_order_detail_returns_canonical_order_view -- --nocapture`

## Result

- outward commerce catalog/quote/order no longer pretend every governed offer is `version=1`
- planned pricing revisions can now surface as outward `draft` publication state before activation
- active pricing revisions now win over newer planned revisions, which matches current commercial live-now semantics

## Architecture Backwrite

- checked architecture doc `166`
- no text change required this loop; implementation now catches up to the existing pricing-lifecycle-governs-outward-commercial-truth direction

## Next Gate

- continue `S01` by removing manual dependence on loosely named pricing `plan_code`
- next best slice: define/administer canonical commercial pricing-plan codes and publication-governance ownership so market/admin/portal all share one explicit revision truth
