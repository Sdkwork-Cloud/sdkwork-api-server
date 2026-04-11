# 2026-04-10 Commercial S03 Admin Marketing Lifecycle Audit Review

## Scope

- Architecture reference: architecture doc `166`
- Step reference: step doc `105`
- Loop focus: close durable, queryable lifecycle audit for coupon-first admin marketing control

## Findings

### P1 - coupon template lifecycle evidence was response-only and not durably queryable

- admin could execute semantic template lifecycle control, but the resulting operator evidence was not persisted behind a dedicated audit surface
- impact:
  - template governance lacked post-hoc evidence beyond the mutation response
  - rejected lifecycle decisions disappeared after the write call returned

### P1 - marketing campaign lifecycle evidence was response-only and not durably queryable

- campaign lifecycle already exposed semantic `publish / schedule / retire`, but no dedicated persisted audit trail existed per campaign
- impact:
  - campaign control remained weaker than commercial publication mutation audit
  - operators could not inspect a stable decision history for active commercial campaigns

### P2 - admin contract lagged lifecycle audit read parity

- backend control had lifecycle mutation audit payloads, but admin shared types and API client still lacked dedicated lifecycle-audit list methods
- impact:
  - frontend-facing control would drift into a second, weaker truth
  - operators would need custom joins or ad-hoc transport code to read audit evidence

## Fix Closure

- added `CouponTemplateLifecycleAuditRecord` and `MarketingCampaignLifecycleAuditRecord` with explicit action and outcome modeling
- persisted audit for both applied and rejected lifecycle decisions in sqlite and postgres
- bound audit fields to runtime governance truth:
  - `operator_id = claims.sub`
  - `request_id = RequestId`
  - `reason = request.reason`
  - `decision_reasons = lifecycle decision blockers`
  - `requested_at_ms = request-time timestamp`
- added dedicated admin read APIs for template and campaign lifecycle audits
- synchronized OpenAPI, admin shared types, and admin API client with audit-list schemas and helpers
- repaired the stale admin marketing API surface expectation count so the test suite matches the expanded contract

## Verification

- RED:
  - new lifecycle-audit regressions failed because storage contracts, routes, schemas, and list APIs were missing
  - admin marketing API surface coverage also failed on stale request-count expectations after the contract expansion
- GREEN:
  - `cargo test -p sdkwork-api-interface-admin --test marketing_coupon_routes admin_marketing_coupon_template_ -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test marketing_coupon_routes admin_marketing_campaign_ -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route openapi_routes_expose_admin_api_inventory_with_schema_components -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin -- --nocapture`
  - `node ./apps/sdkwork-router-admin/tests/admin-marketing-api-surface.test.mjs`
  - `node ./apps/sdkwork-router-admin/tests/admin-marketing-workbench.test.mjs`

## Residual Risks

- `budget / code` surfaces still lag template/campaign lifecycle governance and audit maturity
- template/campaign governance still lacks `revision / approval / compare / clone`
- broader `S03` commercialization closure still needs runtime coupon-first convergence outside this admin audit slice

## Exit

- Step result: `conditional-go`
- Reason:
  - template/campaign lifecycle audit is now durable and queryable
  - `S03` still requires the remaining marketing governance and runtime convergence slices
