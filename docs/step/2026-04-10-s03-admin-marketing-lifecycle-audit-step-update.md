# 2026-04-10 S03 Admin Marketing Lifecycle Audit Step Update

## Scope

- Step: `S03 Coupon-First marketing model convergence`
- Loop target: close durable, queryable lifecycle audit for canonical coupon template and marketing campaign control
- Boundaries:
  - `sdkwork-api-domain-marketing`
  - `sdkwork-api-storage-core`
  - `sdkwork-api-storage-sqlite`
  - `sdkwork-api-storage-postgres`
  - `sdkwork-api-interface-admin`
  - admin shared TS types and admin API client

## Changes

- Domain audit model:
  - added explicit lifecycle action, outcome, and audit record types for coupon template and marketing campaign control
  - persisted `audit_id`, `operator_id`, `request_id`, `reason`, `decision_reasons`, and `requested_at_ms`
  - added `as_str()` and `FromStr` helpers so lifecycle audit enums stay stable across storage and OpenAPI boundaries
- Storage and schema:
  - added insert/list contracts plus per-template and per-campaign audit query methods in admin and marketing store traits
  - added durable sqlite/postgres tables and indexes:
    - `ai_marketing_coupon_template_lifecycle_audit`
    - `ai_marketing_campaign_lifecycle_audit`
  - persisted indexed governance fields plus `record_json` so the audit surface is queryable without freezing future schema evolution
- Admin control plane:
  - template and campaign lifecycle mutations now persist audit records for both applied and rejected decisions
  - lifecycle writes bind `operator_id = claims.sub`, `request_id = RequestId`, and require non-empty lifecycle reason
  - added admin read APIs:
    - `GET /admin/marketing/coupon-templates/{coupon_template_id}/lifecycle-audits`
    - `GET /admin/marketing/campaigns/{marketing_campaign_id}/lifecycle-audits`
- Shared contract:
  - exposed `CouponTemplateLifecycleAuditRecord` and `MarketingCampaignLifecycleAuditRecord` through OpenAPI
  - synchronized admin shared TS types and admin API client with audit-list methods

## Verification

- RED:
  - template and campaign lifecycle evidence was still response-only before this slice; no dedicated audit read APIs or persisted audit tables existed
  - admin marketing API surface coverage also lagged the expanded lifecycle API count after audit-list endpoints were added
- GREEN:
  - `cargo test -p sdkwork-api-interface-admin --test marketing_coupon_routes admin_marketing_coupon_template_ -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test marketing_coupon_routes admin_marketing_campaign_ -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route openapi_routes_expose_admin_api_inventory_with_schema_components -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin -- --nocapture`
  - `node ./apps/sdkwork-router-admin/tests/admin-marketing-api-surface.test.mjs`
  - `node ./apps/sdkwork-router-admin/tests/admin-marketing-workbench.test.mjs`

## Result

- template and campaign lifecycle decisions are now durably persisted instead of living only in mutation responses
- admin can query per-template and per-campaign audit trails without reconstructing operator evidence from write-path logs
- coupon-first marketing control now matches the earlier publication mutation-audit bar on operator traceability and audit readback

## Architecture Backwrite

- updated architecture doc `166`
- corrected repo-truth wording:
  - template and campaign lifecycle audit is now durable and queryable
  - the next governance gap is `budget / code`, plus `revision / approval / compare / clone`

## Next Gate

- `S03` remains open, but template/campaign lifecycle audit persistence and queryability is closed
- next best slice:
  - promote `budget / code` to semantic lifecycle plus durable audit/query
  - or close `revision / approval / compare / clone` for template/campaign governance
