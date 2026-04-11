# 2026-04-10 Commercial S01 Publication-Revision Governance Review

## Scope

- Architecture reference: architecture doc `166`
- Step reference: step doc `103`
- Loop focus: separate publication revision evidence from pricing-plan identity while keeping compatibility

## Findings

### P0 - `CatalogPublication` only exposed status-shaped evidence

- previous publication truth stopped at `publication_id / publication_kind / publication_status`
- impact:
  - portal could not tell which publication revision was active
  - later admin publication control would have to overload `pricing_plan_version`

### P0 - quote and order snapshot could not freeze publication revision identity

- order snapshot carried owner-chain and pricing binding, but not explicit publication revision evidence
- impact:
  - planned/published cutover evidence stayed ambiguous in commerce audit trails

### P1 - portal OpenAPI exposed publication ownership but not publication governance evidence

- catalog offers and order views already surfaced publication owner chain
- impact:
  - outward contract still hid revision/source/effective-window semantics needed for professional publication governance

## Fix Closure

- upgraded `CatalogPublication` with explicit revision id, version, source kind, and effective-from evidence
- centralized publication governance derivation in app-catalog from either governed pricing-plan truth or catalog-seed fallback truth
- threaded additive publication revision evidence through commerce catalog binding, quote, order snapshot, settlement reconstruction, portal catalog, portal order views, and portal OpenAPI
- preserved existing `pricing_plan_*` contract so publication governance and pricing governance are related but no longer conflated

## Verification

- RED:
  - app-catalog publication-governance assertions failed because revision evidence fields did not exist
  - app-commerce quote regression failed because quote/snapshot did not expose publication revision evidence
  - portal catalog and OpenAPI regressions failed because outward publication governance fields were absent
- GREEN:
  - `cargo test -p sdkwork-api-domain-catalog --test commercial_catalog -- --nocapture`
  - `cargo test -p sdkwork-api-app-catalog --test commercial_catalog -- --nocapture`
  - `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test openapi_route -- --nocapture`

## Residual Risks

- publication governance is still projected from pricing-plan lifecycle, not yet an independent admin-managed record
- `public_api` and other publication channels still do not have channel-specific governed revisions
- admin control plane still lacks publication list/detail/revision actions

## Exit

- Step result: `conditional-go`
- Reason:
  - outward publication revision semantics are now explicit and audit-safe
  - S01 still needs admin publication control before publication governance can be called first-class
