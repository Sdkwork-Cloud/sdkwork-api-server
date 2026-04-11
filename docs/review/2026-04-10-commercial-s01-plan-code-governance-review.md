# 2026-04-10 Commercial S01 Plan-Code Governance Review

## Scope

- Architecture reference: architecture doc `166`
- Step reference: step doc `103`
- Loop focus: eliminate free-form commercial pricing-code drift between admin pricing CRUD and runtime catalog governance

## Findings

### P0 - admin pricing still depended on raw operator-entered `plan_code`

- pricing create/update accepted any trimmed string
- impact:
  - canonical commercial pricing alignment still depended on manual string discipline
  - admin could create product-bound pricing rows that runtime catalog could not recognize

### P0 - catalog governance still matched commercial pricing by exact stored string

- pricing-governance selection required exact equality with `<product_kind>:<target_id>`
- impact:
  - historical variants such as `Subscription-Plan : growth` were invisible to canonical catalog governance

### P1 - compatibility still had to preserve generic non-commercial plan codes

- pricing control plane is broader than product-bound market pricing
- impact:
  - hard-switching every `plan_code` to commercial semantics would break generic billing use cases

## Fix Closure

- introduced shared `normalize_commercial_pricing_plan_code(...)` helper in app-catalog
- normalized commercial product-kind variants into canonical snake-case `<product_kind>:<target_id>`
- rejected malformed commercial codes that declare a commercial product kind but omit `target_id`
- upgraded catalog governance matching to use the shared helper instead of raw string equality
- upgraded admin pricing create/update to persist normalized commercial codes while leaving generic codes unchanged

## Verification

- RED:
  - app-catalog commercial-code normalization test failed because the shared helper did not exist
  - admin pricing normalization regression failed because create/update persisted raw `Subscription-Plan : growth`
- GREEN:
  - `cargo test -p sdkwork-api-app-catalog --test commercial_catalog -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test account_billing_routes admin_billing_pricing_management_routes_normalize_commercial_plan_codes_on_create_and_update -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test account_billing_routes admin_billing_pricing_management_routes_reject_commercial_plan_codes_without_target_id -- --nocapture`

## Residual Risks

- `CatalogPublication` is still derived from pricing status instead of being an admin-governed revision entity
- admin UI still exposes `plan_code` as free text; semantics are now safer, but not yet first-class in the operator form model
- generic pricing plans and commercial product-bound pricing still share one storage record type

## Exit

- Step result: `conditional-go`
- Reason:
  - canonical commercial pricing-code truth is now shared across admin write path and catalog read path
  - S01 still needs publication revision ownership to reach full commercial-governed closure
