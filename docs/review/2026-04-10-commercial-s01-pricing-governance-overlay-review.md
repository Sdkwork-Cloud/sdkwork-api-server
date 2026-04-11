# 2026-04-10 Commercial S01 Pricing Governance Overlay Review

## Scope

- Architecture reference: architecture doc `166`
- Step reference: step doc `103`
- Loop focus: make outward commercial catalog/quote/order read governed pricing-plan version and publication state instead of builder-only defaults

## Findings

### P0 - outward commercial pricing evidence was still builder-derived

- canonical catalog builder always emitted:
  - `pricing_plan_version = 1`
  - `publication_status = published`
- impact:
  - active pricing lifecycle truth could not reach outward catalog/quote/order
  - planned or archived pricing revisions were invisible at the commercial contract edge

### P0 - commerce could not safely read pricing governance through the shared store abstraction

- app-commerce only received `&dyn AdminStore`
- `AccountKernelStore` existed, but the trait-object boundary hid it
- impact:
  - live pricing-plan governance could not be consulted without widening many app/router signatures

### P1 - governance selection needed current-live precedence, not naive latest-version precedence

- a newer `planned` revision should not replace a still-`active` live offer in outward catalog truth
- impact:
  - naive max-version selection would misstate what is actually purchasable now

## Fix Closure

- introduced deterministic commercial `plan_code` convention `<product_kind>:<target_id>`
- overlaid catalog `pricing_plan_version / publication_status` from aligned pricing-plan records
- selected governance with explicit precedence:
  - `active/published`
  - `planned/draft`
  - `retired/archived`
- added `AdminStore::account_kernel_store()` bridge and wired sqlite/postgres implementations
- routed portal catalog and quote generation through store-backed canonical catalog construction
- verified quote snapshot and portal outward contract remain compatible when no aligned plan exists

## Verification

- RED:
  - app-commerce quote governance-version regression failed with `1 != 3`
  - app-commerce planned-only governance regression failed with `1 != 4`
  - portal catalog governance regression failed with outward offer `pricing_plan_version = 1`
- GREEN:
  - `cargo test -p sdkwork-api-app-catalog --test commercial_catalog -- --nocapture`
  - `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure quote_preview -- --nocapture`
  - `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure submitted_order_freezes_quote_pricing_binding_in_snapshot -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_catalog_exposes_plans_packs_and_active_coupons -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_quote_prices_recharge_and_coupon_redemption -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_order_detail_returns_canonical_order_view -- --nocapture`

## Residual Risks

- alignment still depends on disciplined commercial `plan_code` naming; non-aligned pricing records are intentionally ignored
- outward `pricing_plan_id` is still compatibility-oriented string identity, not the numeric billing plan primary key
- `CatalogPublication` is still not an admin-governed first-class publication revision entity

## Exit

- Step result: `conditional-go`
- Reason:
  - S01 outward pricing evidence now follows live governance when aligned truth exists
  - S01 still needs canonical pricing-plan code governance and publication lifecycle ownership to become full `go`
