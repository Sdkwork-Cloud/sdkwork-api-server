# 2026-04-10 S03 Admin Marketing Coupon Template Revision Governance Step Update

## Scope

- Step: `S03 Coupon-First marketing model convergence`
- Loop target: close canonical `coupon template` revision governance on the admin control plane
- Boundaries:
  - `sdkwork-api-domain-marketing`
  - `sdkwork-api-interface-admin`
  - admin shared TS types and admin API client

## Changes

- Model:
  - added additive template governance fields:
    - `approval_state`
    - `revision`
    - `root_coupon_template_id`
    - `parent_coupon_template_id`
- Governance:
  - promoted template revision workflow into semantic admin actions:
    - `clone`
    - `compare`
    - `submit-for-approval`
    - `approve`
    - `reject`
  - kept `publish / schedule / retire` as rollout actions, but gated rollout behind approval for governed revisions
- Audit:
  - extended template lifecycle audit with revision governance evidence:
    - `source_coupon_template_id`
    - `previous_approval_state / resulting_approval_state`
    - `previous_revision / resulting_revision`
- Contract:
  - added admin APIs:
    - `POST /admin/marketing/coupon-templates/{coupon_template_id}/clone`
    - `POST /admin/marketing/coupon-templates/{coupon_template_id}/compare`
    - `POST /admin/marketing/coupon-templates/{coupon_template_id}/submit-for-approval`
    - `POST /admin/marketing/coupon-templates/{coupon_template_id}/approve`
    - `POST /admin/marketing/coupon-templates/{coupon_template_id}/reject`
  - synchronized OpenAPI, admin TS types, and admin API client

## Verification

- RED:
  - new template revision-governance route test failed with `404`
  - OpenAPI inventory test failed because new template governance paths were absent
  - admin marketing API surface test failed because revision-governance types and methods were absent
- GREEN:
  - `cargo test -p sdkwork-api-interface-admin --test marketing_coupon_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route -- --nocapture`
  - `node ./apps/sdkwork-router-admin/tests/admin-marketing-api-surface.test.mjs`

## Result

- template is no longer only a raw `create + status + publish` object; it now supports governed revision branching and approval semantics
- clone produces a new governed draft revision with lineage and revision evidence
- compare exposes structured field-level change review
- template `publish / schedule` no longer bypass approval on governed revisions

## Exit

- Step result: `conditional-go`
- Reason:
  - template revision governance is closed
  - `S03` still has remaining campaign-side revision governance plus wider portal/public/account convergence
