# 2026-04-10 Commercial S01 Admin Publication Projection Review

## Scope

- Architecture reference: architecture doc `166`
- Step reference: step doc `103`
- Loop focus: make canonical commercial publication truth visible on admin without forking storage or semantics

## Findings

### P1 - admin could not inspect publication truth directly

- portal already exposed publication owner/governance evidence, but admin operators still had to infer publication state from pricing plans
- impact:
  - publication review, rollout checking, and future publish controls lacked an explicit operator-facing read model

### P1 - canonical product / offer / publication chain stopped before admin OpenAPI

- the canonical model existed in domain/app layers, but admin contract did not expose a first-class projection
- impact:
  - downstream admin tools could not depend on a stable publication projection schema

## Fix Closure

- reused `current_canonical_commercial_catalog_for_store(...)` as the single read-truth source
- added additive admin route `GET /admin/commerce/catalog-publications`
- returned nested canonical `product / offer / publication` objects instead of flattening into a second admin-specific truth model
- exposed the same projection in admin OpenAPI

## Verification

- RED:
  - route regression failed with `404`
  - OpenAPI regression failed because `/admin/commerce/catalog-publications` and `CommercialCatalogPublicationProjection` were absent
- GREEN:
  - `cargo test -p sdkwork-api-interface-admin admin_commerce_catalog_publications_expose_canonical_product_offer_publication_chain -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin openapi_routes_expose_admin_api_inventory_with_schema_components -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test account_billing_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route -- --nocapture`

## Residual Risks

- admin still has read-only publication projection, not publication actions
- publication governance is still derived from pricing lifecycle, not an independent persisted publication aggregate
- channel-specific publication governance is still absent

## Exit

- Step result: `conditional-go`
- Reason:
  - admin publication visibility is now explicit and reusable
  - S01 still needs publication actions/governance closure before the publication capability can be called complete
