# 2026-04-10 Commercial S03 Admin Marketing Coupon Template Lifecycle Review

## Scope

- Architecture reference: architecture doc `166`
- Step reference: step doc `105`
- Loop focus: close semantic admin coupon-template lifecycle for coupon-first marketing control

## Findings

### P1 - coupon template control still depended on generic status mutation

- admin could create templates and mutate `status`, but could not execute semantic `publish / schedule / retire`
- impact:
  - definition-side control lagged campaign-side control
  - operators had no explicit template activation timing contract

### P1 - template lifecycle lacked additive timing semantics

- `CouponTemplateRecord` had no explicit publish window field
- impact:
  - template readiness could not distinguish “future scheduled” from “immediately publishable”
  - semantic `schedule` could not be modeled without overloading generic status

### P2 - admin shared contract lagged backend lifecycle semantics

- admin/portal types still modeled template status as `draft | active | archived`
- admin API client exposed only `/status`
- impact:
  - frontend contract would drift into a second, older truth even after backend lifecycle upgrade

## Fix Closure

- added `CouponTemplateStatus::Scheduled`
- added additive `activation_at_ms` on `CouponTemplateRecord`
- added semantic template lifecycle handlers and routes:
  - `publish`
  - `schedule`
  - `retire`
- added `PublishCouponTemplateRequest / ScheduleCouponTemplateRequest / RetireCouponTemplateRequest`
- added `CouponTemplateMutationResult { detail, audit }`
- synchronized admin/portal shared types and admin API client methods to the new lifecycle contract
- preserved `/status` for compatibility while shifting new control-plane writes to semantic routes

## Verification

- RED:
  - new template lifecycle regressions failed with `404`
  - OpenAPI regression failed because template lifecycle paths and schemas were missing
- GREEN:
  - `cargo test -p sdkwork-api-interface-admin --test marketing_coupon_routes admin_marketing_coupon_template_ -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route openapi_routes_expose_admin_api_inventory_with_schema_components -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test marketing_coupon_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin -- --nocapture`
  - `node ./apps/sdkwork-router-admin/tests/admin-marketing-api-surface.test.mjs`
  - `node ./apps/sdkwork-router-admin/tests/admin-marketing-workbench.test.mjs`
  - `node ./apps/sdkwork-router-portal/tests/portal-marketing-api-surface.test.mjs`

## Residual Risks

- template lifecycle audit is still response-only, not durably queryable or persisted
- budget/code surfaces still lag template/campaign lifecycle maturity
- template lifecycle still lacks `revision / approval / compare / clone`

## Exit

- Step result: `conditional-go`
- Reason:
  - definition-side semantic lifecycle is now closed enough to support coupon-first control
  - broader `S03` convergence still requires audit persistence and the remaining marketing surfaces
