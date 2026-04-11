# 2026-04-10 S01 Commercial Admin Publication Mutation Audit Step Update

## Scope

- Step: `S01 API product and pricing model convergence`
- Loop target: close admin publication semantic mutations and persistent lifecycle audit
- Boundaries: `sdkwork-api-interface-admin`, publication-audit storage, admin tests

## Changes

- Admin semantic mutation routes:
  - added `POST /admin/commerce/catalog-publications/{publication_id}/publish`
  - added `POST /admin/commerce/catalog-publications/{publication_id}/schedule`
  - added `POST /admin/commerce/catalog-publications/{publication_id}/retire`
- Response contract:
  - added `CommercialCatalogPublicationMutationResult { detail, audit }`
  - mutation readback now returns canonical publication detail plus persisted lifecycle audit evidence
- Audit persistence:
  - persisted `CatalogPublicationLifecycleAuditRecord`
  - recorded `action / outcome / operator_id / request_id / operator_reason / before-after status / decision_reasons`
- Lifecycle semantics:
  - reused publication `actionability` as write-side preflight truth
  - `catalog_seed` publication write attempts are rejected and still audited
  - governed `planned` pricing now blocks repeat `schedule` with explicit reason `publication is already scheduled`
  - `publish / schedule / retire` now map to governed pricing-plan and pricing-rate status transitions
- Verification hardening:
  - fixed stale sqlite admin integration test precondition by creating `provider-model` before `model-price`, matching current admin pricing contract

## Verification

- RED:
  - `cargo test -p sdkwork-api-interface-admin --test account_billing_routes admin_commerce_catalog_publication_publish_mutation_updates_governed_publication_and_records_audit -- --nocapture`
- GREEN:
  - `cargo test -p sdkwork-api-interface-admin --test account_billing_routes commerce_catalog -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes builtin_channels_channel_models_and_model_prices_are_exposed_through_admin_api -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin -- --nocapture`

## Result

- admin publication control is now semantic on both read and write sides
- publication lifecycle decisions and operator intent are durably auditable instead of response-only
- canonical catalog outward semantics stay stable while governed pricing remains the internal enforcement truth

## Architecture Backwrite

- checked architecture doc `166`
- no architecture text change required this loop; implementation now matches the semantic publication-control and governed-pricing write-side direction

## Next Gate

- `S01` remains open but this slice is closed
- next best slice:
  - add publication lifecycle audit list/read APIs if operator evidence retrieval is required immediately
  - otherwise move to next unlocked step window `S02 || S03`
