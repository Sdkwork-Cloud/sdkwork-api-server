# 2026-04-10 Commercial S01 Admin Publication Detail Review

## Scope

- Architecture reference: architecture doc `166`
- Step reference: step doc `103`
- Loop focus: expose governed pricing context behind a canonical publication detail

## Findings

### P1 - publication list was visible, but governed pricing context was still implicit

- operators could see `product / offer / publication`, but not the exact pricing-plan record and rates backing the current publication revision
- impact:
  - publication review still required manual inference from pricing screens
  - later publication lifecycle actions would have lacked a stable detail contract

### P1 - admin OpenAPI stopped at publication projection, not publication detail

- downstream admin consumers had no contract for loading one publication with governance context
- impact:
  - integration would drift toward ad-hoc multi-call joining on the client side

## Fix Closure

- added additive `GET /admin/commerce/catalog-publications/{publication_id}`
- introduced `CommercialCatalogPublicationDetail { projection, governed_pricing_plan, governed_pricing_rates }`
- resolved governed pricing truth through canonical commercial code plus publication version instead of ad-hoc id guessing
- kept `catalog_seed` publications readable without pretending they are already governed by a pricing plan

## Verification

- RED:
  - detail route regression failed with `404`
  - OpenAPI regression failed because detail route/schema were absent
- GREEN:
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test account_billing_routes -- --nocapture`

## Residual Risks

- publication lifecycle actions are still missing
- action audit metadata such as `operator / reason / request_id` is not yet modeled for publication operations
- channel-aware publication governance is still absent

## Exit

- Step result: `conditional-go`
- Reason:
  - admin detail semantics are now strong enough for professional operator inspection
  - S01 still needs explicit publication actions before the commercial publication control plane is complete
