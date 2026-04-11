# 2026-04-10 S03 Admin Marketing Coupon Template Lifecycle Step Update

## Scope

- Step: `S03 Coupon-First marketing model convergence`
- Loop target: replace coupon-template-side generic status mutation with semantic `publish / schedule / retire`
- Boundaries:
  - `sdkwork-api-domain-marketing`
  - `sdkwork-api-interface-admin`
  - admin shared TS types and admin API client

## Changes

- Domain lifecycle:
  - added `CouponTemplateStatus::Scheduled`
  - added additive `activation_at_ms` on `CouponTemplateRecord`
- Admin semantic lifecycle routes:
  - added `POST /admin/marketing/coupon-templates/{coupon_template_id}/publish`
  - added `POST /admin/marketing/coupon-templates/{coupon_template_id}/schedule`
  - added `POST /admin/marketing/coupon-templates/{coupon_template_id}/retire`
- Response contract:
  - added `CouponTemplateMutationResult { detail, audit }`
  - mutation readback now returns `coupon_template + actionability`, not only a raw status change
- Lifecycle semantics:
  - `publish` rejects future `activation_at_ms`
  - `schedule` requires future `activation_at_ms`
  - `retire` converges template to `archived`
- Shared contract:
  - synchronized admin/portal `CouponTemplateStatus` and `CouponTemplateRecord.activation_at_ms`
  - added admin API client methods for template `publish / schedule / retire`

## Verification

- RED:
  - `cargo test -p sdkwork-api-interface-admin --test marketing_coupon_routes admin_marketing_coupon_template_ -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route openapi_routes_expose_admin_api_inventory_with_schema_components -- --nocapture`
- GREEN:
  - `cargo test -p sdkwork-api-interface-admin --test marketing_coupon_routes admin_marketing_coupon_template_ -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route openapi_routes_expose_admin_api_inventory_with_schema_components -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test marketing_coupon_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin -- --nocapture`
  - `node ./apps/sdkwork-router-admin/tests/admin-marketing-api-surface.test.mjs`
  - `node ./apps/sdkwork-router-admin/tests/admin-marketing-workbench.test.mjs`
  - `node ./apps/sdkwork-router-portal/tests/portal-marketing-api-surface.test.mjs`

## Result

- coupon definition control is now semantic on the write side, not only campaign control
- template activation timing is now explicit through `activation_at_ms`
- admin client and shared types no longer lag behind backend lifecycle truth

## Architecture Backwrite

- updated architecture doc `166`
- corrected repo-truth wording:
  - marketing is no longer only `create + status toggle`
  - template/campaign semantic lifecycle is closed; budget/code remain the next control gap

## Next Gate

- `S03` remains open but template lifecycle slice is closed
- next best slice:
  - promote budget/code to semantic lifecycle or governed actionability
  - or start durable lifecycle audit persistence/query instead of response-only evidence
