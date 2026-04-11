# 2026-04-10 S03 Admin Marketing Budget/Code Lifecycle Step Update

## Scope

- Step: `S03 Coupon-First marketing model convergence`
- Loop target: close semantic lifecycle plus durable audit/query for canonical `campaign budget` and `coupon code`
- Boundaries:
  - `sdkwork-api-domain-marketing`
  - `sdkwork-api-storage-core`
  - `sdkwork-api-storage-sqlite`
  - `sdkwork-api-storage-postgres`
  - `sdkwork-api-interface-admin`
  - admin shared TS types and admin API client

## Changes

- Semantic lifecycle:
  - budget primary admin actions are now `activate / close`
  - code primary admin actions are now `disable / restore`
  - legacy `/status` routes stay for compatibility only
- Governance model:
  - added `CampaignBudgetActionability / Detail / MutationResult`
  - added `CouponCodeActionability / Detail / MutationResult`
  - persisted applied and rejected lifecycle audit for both aggregates
- Guardrails:
  - budget `activate` rejects `closed`, already `active`, zero headroom, and ended/archived campaign context
  - code `restore` rejects expired code
  - code operator lifecycle no longer treats runtime-owned `reserved / redeemed` as normal admin toggle states
- Contract:
  - added admin APIs:
    - `POST /admin/marketing/budgets/{campaign_budget_id}/activate`
    - `POST /admin/marketing/budgets/{campaign_budget_id}/close`
    - `GET /admin/marketing/budgets/{campaign_budget_id}/lifecycle-audits`
    - `POST /admin/marketing/codes/{coupon_code_id}/disable`
    - `POST /admin/marketing/codes/{coupon_code_id}/restore`
    - `GET /admin/marketing/codes/{coupon_code_id}/lifecycle-audits`
  - synchronized OpenAPI, admin TS types, and admin API client

## Verification

- RED:
  - new budget/code lifecycle route tests failed with `404`
  - admin marketing API surface test failed because lifecycle types and client methods were missing
- GREEN:
  - `cargo test -p sdkwork-api-interface-admin --test marketing_coupon_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route openapi_routes_expose_admin_api_inventory_with_schema_components -- --nocapture`
  - `node ./apps/sdkwork-router-admin/tests/admin-marketing-api-surface.test.mjs`

## Result

- budget/code lifecycle is no longer primarily a generic status toggle surface
- operator evidence for budget/code control is now durable and queryable
- `S03` next governance gap narrows to `revision / approval / compare / clone`

## Exit

- Step result: `conditional-go`
- Reason:
  - budget/code semantic lifecycle plus audit/query is closed
  - wider `S03` commercialization convergence is still not finished
