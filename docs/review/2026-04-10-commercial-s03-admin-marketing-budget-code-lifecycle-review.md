# 2026-04-10 Commercial S03 Admin Marketing Budget/Code Lifecycle Review

## Scope

- Architecture reference: architecture doc `166`
- Step reference: step doc `105`
- Loop focus: close semantic lifecycle governance for canonical `campaign budget` and `coupon code`

## Findings

### P1 - budget governance still depended on raw status toggle

- admin only had `/status` for budget mutation, so intent and guardrails were hidden behind a generic `CampaignBudgetStatus`
- impact:
  - operators could not distinguish `activate` from arbitrary status overwrite
  - budget lifecycle had no dedicated durable audit/query surface

### P1 - coupon code governance still depended on raw status toggle

- admin only had `/status` for code mutation, which made `disable / restore` semantics implicit and encouraged writes into runtime-owned states
- impact:
  - expired code could be incorrectly restored to `available`
  - `reserved / redeemed` could drift into operator-managed toggle semantics

### P2 - admin contract lacked budget/code lifecycle parity

- template/campaign already had semantic lifecycle and lifecycle-audit APIs, but budget/code still lagged in OpenAPI, TS types, and admin client helpers
- impact:
  - admin frontend contract would stay uneven
  - governance capability would fragment by aggregate

## Fix Closure

- added semantic lifecycle routes for budget `activate / close` and code `disable / restore`
- added additive detail, actionability, mutation-result, and lifecycle-audit models for both aggregates
- persisted both applied and rejected lifecycle decisions with:
  - `operator_id = claims.sub`
  - `request_id = RequestId`
  - `reason = request.reason`
  - `decision_reasons = governance blockers`
  - `requested_at_ms = request-time timestamp`
- enforced explicit rules:
  - budget activate rejects `closed`, already `active`, zero headroom, and ended/archived campaign context
  - code restore rejects expired code
  - runtime-owned `reserved / redeemed` no longer behave like normal operator toggles
- synchronized backend routes, OpenAPI, admin TS types, and admin API client

## Verification

- RED:
  - lifecycle route regressions failed with `404`
  - admin marketing API surface failed because budget/code lifecycle types and methods did not exist
- GREEN:
  - `cargo test -p sdkwork-api-interface-admin --test marketing_coupon_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route openapi_routes_expose_admin_api_inventory_with_schema_components -- --nocapture`
  - `node ./apps/sdkwork-router-admin/tests/admin-marketing-api-surface.test.mjs`

## Residual Risks

- template/campaign governance still lacks `revision / approval / compare / clone`
- broader `S03` portal/public/account convergence is still open

## Exit

- Step result: `conditional-go`
- Reason:
  - budget/code governance parity is now closed
  - `S03` still has remaining commercialization slices
