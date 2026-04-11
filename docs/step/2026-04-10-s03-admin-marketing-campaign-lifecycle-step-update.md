# 2026-04-10 S03 Admin Marketing Campaign Lifecycle Step Update

## Scope

- Step: `S03 Coupon-First marketing model convergence`
- Loop target: replace generic marketing campaign status toggle with semantic `publish / schedule / retire`
- Boundaries: `sdkwork-api-interface-admin` marketing admin routes, OpenAPI, route tests

## Changes

- Admin semantic lifecycle routes:
  - added `POST /admin/marketing/campaigns/{marketing_campaign_id}/publish`
  - added `POST /admin/marketing/campaigns/{marketing_campaign_id}/schedule`
  - added `POST /admin/marketing/campaigns/{marketing_campaign_id}/retire`
- Response contract:
  - added `MarketingCampaignMutationResult { detail, audit }`
  - mutation readback now returns `campaign + coupon_template + actionability`, not only a raw status change
- Audit readback:
  - added response-side `MarketingCampaignLifecycleAuditRecord`
  - recorded `action / outcome / previous_status / resulting_status / reason / requested_at_ms`
- Lifecycle semantics:
  - `publish` requires active coupon template, non-closed campaign, non-future `start_at_ms`, and non-expired `end_at_ms`
  - `schedule` requires active coupon template and future `start_at_ms`
  - `retire` maps campaign to `ended`
- Compatibility:
  - retained legacy `/status` mutation for compatibility
  - positioned semantic routes as the new write-side contract for coupon-first operations

## Verification

- RED:
  - `cargo test -p sdkwork-api-interface-admin --test marketing_coupon_routes admin_marketing_campaign_ -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route openapi_routes_expose_admin_api_inventory_with_schema_components -- --nocapture`
- GREEN:
  - `cargo test -p sdkwork-api-interface-admin --test marketing_coupon_routes admin_marketing_campaign_ -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route openapi_routes_expose_admin_api_inventory_with_schema_components -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test marketing_coupon_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin -- --nocapture`

## Result

- admin marketing campaign control is now semantic on the write side
- coupon template state is now part of campaign mutation truth, reducing generic marketing drift
- future campaign publish attempts now fail with explicit lifecycle reason instead of leaking silent state misuse

## Architecture Backwrite

- checked architecture doc `166` and step doc `105`
- no architecture text change required this loop; implementation now closes the `create + status toggle` gap for campaign control

## Next Gate

- `S03` remains open but this slice is closed
- next best slice:
  - add coupon template semantic lifecycle so definition-side control matches campaign-side control
  - or promote issued-coupon / my-coupons semantics so coupon ownership is no longer code-centric
