# 2026-04-10 S01 Commercial Admin Publication Actionability Step Update

## Scope

- Step: `S01 API product and pricing model convergence`
- Loop target: expose operator-ready publication lifecycle actionability on canonical admin publication detail
- Boundaries: `sdkwork-api-interface-admin`

## Changes

- Admin publication detail:
  - kept `GET /admin/commerce/catalog-publications/{publication_id}` as the single read-side publication control entry
  - added `actionability.publish / schedule / retire`
- Actionability semantics:
  - `catalog_seed` publication: all actions blocked, reason explicit
  - governed `draft` publication with future `effective_from_ms`: `publish` blocked, `schedule` allowed
  - governed `draft` publication with current/past `effective_from_ms`: `publish` allowed, `schedule` blocked
  - `published` publication: `publish` and `schedule` blocked
  - `archived` publication: all re-publish semantics blocked; `retire` blocked as already retired
  - no governed pricing rates: lifecycle actions blocked with explicit reason
- Regression hardening:
  - unified publication tests on real future timestamps instead of stale `20ms` expectations
  - asserted governed plan and publication effective time consistency in detail response

## Verification

- RED:
  - `cargo test -p sdkwork-api-interface-admin admin_commerce_catalog_publications_expose_canonical_product_offer_publication_chain -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin admin_commerce_catalog_publication_detail_exposes_governed_pricing_context -- --nocapture`
- GREEN:
  - `cargo test -p sdkwork-api-interface-admin --test account_billing_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route -- --nocapture`

## Result

- admin can now inspect one publication and immediately know whether to publish, schedule, or retire without re-deriving pricing lifecycle rules mentally
- publication control stays read-side and additive; canonical catalog plus governed pricing remain the only truth sources
- future semantic mutation endpoints can now reuse stable action gating instead of duplicating lifecycle rules in clients

## Architecture Backwrite

- checked architecture doc `166`
- no text change required this loop; implementation now matches the admin publication control and operator-semantic direction more closely

## Next Gate

- continue `S01`
- next best slice: add semantic admin publication mutation endpoints with audit fields `operator / reason / request_id` and reuse current `actionability` as preflight truth
