# 2026-04-10 Commercial S03 Admin Marketing Campaign Lifecycle Review

## Scope

- Architecture reference: architecture doc `166`
- Step reference: step doc `105`
- Loop focus: close semantic admin campaign lifecycle for coupon-first marketing control

## Findings

### P1 - admin marketing campaign control was still generic status toggle

- admin could create campaign records and mutate `status`, but could not execute semantic `publish / schedule / retire`
- impact:
  - operator workflow still depended on generic status knowledge instead of coupon-first lifecycle semantics
  - timing rules around `start_at_ms` remained implicit and easy to misuse

### P1 - campaign write responses lacked explicit coupon semantic context

- admin mutations returned only `MarketingCampaignRecord`, without linked coupon template truth or actionability readback
- impact:
  - clients had to join coupon definition state separately
  - write-side tooling could drift toward generic marketing rather than explicit coupon control

### P2 - admin OpenAPI did not expose semantic campaign lifecycle surface

- admin schema inventory had no canonical contract for campaign `publish / schedule / retire`
- impact:
  - generated clients could not adopt semantic lifecycle actions
  - contract evolution would keep favoring legacy `/status`

## Fix Closure

- added semantic campaign lifecycle handlers and routes:
  - `publish`
  - `schedule`
  - `retire`
- added `PublishMarketingCampaignRequest / ScheduleMarketingCampaignRequest / RetireMarketingCampaignRequest`
- added `MarketingCampaignMutationResult { detail, audit }`
- bound lifecycle preflight to coupon-first truth:
  - coupon template must be active
  - future `start_at_ms` blocks `publish` and requires `schedule`
  - closed campaigns cannot be re-published or re-scheduled
- preserved `/status` for compatibility while shifting new clients to semantic routes

## Verification

- RED:
  - new route regressions failed with `404`
  - OpenAPI regression failed because semantic campaign lifecycle paths and schemas were missing
- GREEN:
  - `cargo test -p sdkwork-api-interface-admin --test marketing_coupon_routes admin_marketing_campaign_ -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route openapi_routes_expose_admin_api_inventory_with_schema_components -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test marketing_coupon_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin -- --nocapture`

## Residual Risks

- campaign lifecycle audit is currently response-only, not durably queryable or persisted
- coupon template, budget, and code lifecycles still use generic status toggles
- outward route remains `/admin/marketing/campaigns`; dedicated coupon-campaign aliasing is not yet introduced

## Exit

- Step result: `conditional-go`
- Reason:
  - campaign write-side semantics now match coupon-first lifecycle intent
  - broader `S03` convergence still requires definition-side and ownership-side completion
