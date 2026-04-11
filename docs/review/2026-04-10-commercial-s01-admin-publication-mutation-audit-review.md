# 2026-04-10 Commercial S01 Admin Publication Mutation Audit Review

## Scope

- Architecture reference: architecture doc `166`
- Step reference: step doc `103`
- Loop focus: close semantic admin publication mutations with durable audit evidence

## Findings

### P1 - admin publication control was still read-only

- admin could inspect publication actionability, but could not execute semantic `publish / schedule / retire` from the canonical publication surface
- impact:
  - operator workflow still leaked internal pricing-plan ids
  - write-side clients would have duplicated lifecycle rules outside the publication contract

### P1 - publication lifecycle audit was not durable

- operator reason, request id, and lifecycle decision outcome were not persisted for publication control actions
- impact:
  - publication governance lacked post-hoc evidence
  - rejected lifecycle attempts would disappear after the response

### P2 - package verification still carried a stale admin pricing test precondition

- one sqlite admin integration test still assumed `model-price` creation without a prior `provider-model`
- impact:
  - full-package verification failed even though the enforced contract already had a dedicated regression proving the stricter prerequisite

## Fix Closure

- added semantic admin publication mutation handlers and OpenAPI exposure
- added `CommercialCatalogPublicationMutationResult { detail, audit }`
- persisted `CatalogPublicationLifecycleAuditRecord` for both applied and rejected actions
- bound audit source fields to real runtime truth:
  - `operator_id = claims.sub`
  - `request_id = RequestId`
  - `recorded_at_ms = unix_timestamp_ms()`
  - `operator_reason = request.reason`
- hardened actionability so governed `planned` pricing blocks repeat `schedule`
- aligned the stale sqlite integration test to the current `provider-model -> model-price` prerequisite

## Verification

- RED:
  - missing publication mutation handlers and response schema caused compile failure on new publication mutation regressions
  - full-package verification exposed stale sqlite admin test setup at `builtin_channels_channel_models_and_model_prices_are_exposed_through_admin_api`
- GREEN:
  - `cargo test -p sdkwork-api-interface-admin --test account_billing_routes commerce_catalog -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes builtin_channels_channel_models_and_model_prices_are_exposed_through_admin_api -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin -- --nocapture`

## Residual Risks

- admin still lacks publication lifecycle audit query APIs
- publication governance is still not channel-aware
- broader `S01` convergence is not yet wave-complete

## Exit

- Step result: `conditional-go`
- Reason:
  - semantic publication mutations and durable audit are now implemented and verified
  - `S01` still has follow-on convergence work outside this publication-control slice
