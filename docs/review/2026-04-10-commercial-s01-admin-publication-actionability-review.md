# 2026-04-10 Commercial S01 Admin Publication Actionability Review

## Scope

- Architecture reference: architecture doc `166`
- Step reference: step doc `103`
- Loop focus: make admin publication detail operator-actionable without adding unsafe mutations first

## Findings

### P1 - governed publication detail still required operator-side lifecycle inference

- detail exposed governed pricing evidence, but not whether the current publication could be published, scheduled, or retired
- impact:
  - operator decisions still depended on hidden pricing lifecycle knowledge
  - later mutation endpoints would likely duplicate gating logic across clients

### P1 - publication regressions still carried stale pseudo-future time semantics

- publication tests mixed real future timestamps with legacy `20ms` expectations
- impact:
  - future-effective draft publication behavior was asserted incorrectly
  - control-plane semantics could drift silently

## Fix Closure

- added `CommercialCatalogPublicationActionability` and `CommercialCatalogPublicationActionDecision`
- attached `actionability` to `CommercialCatalogPublicationDetail`
- blocked or allowed `publish / schedule / retire` from canonical publication status, governed pricing presence, governed rate presence, and effective time
- hardened regressions to use real future timestamps and explicit effective-time assertions

## Verification

- RED:
  - publication list regression failed because `publication_effective_from_ms` expected stale `20`
  - publication detail regression failed because future-effective draft actionability was asserted against stale test data
- GREEN:
  - `cargo test -p sdkwork-api-interface-admin --test account_billing_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route -- --nocapture`

## Residual Risks

- admin still lacks semantic publication mutation endpoints
- publication action audit fields are not modeled yet
- channel-aware publication governance is still absent

## Exit

- Step result: `conditional-go`
- Reason:
  - operator-facing publication lifecycle semantics are now explicit and test-backed
  - S01 still needs audited semantic mutations before the publication control plane is commercially complete
