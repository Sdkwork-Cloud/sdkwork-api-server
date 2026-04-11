# 2026-04-10 S03 Admin Marketing Campaign Revision Governance Step Update

## Scope

- Step: `S03 Coupon-First marketing model convergence`
- Loop target: close canonical `marketing campaign` revision governance on the admin control plane
- Boundaries:
  - `sdkwork-api-domain-marketing`
  - `sdkwork-api-interface-admin`
  - admin shared TS types and admin API client

## Changes

- Model:
  - added additive campaign governance fields:
    - `approval_state`
    - `revision`
    - `root_marketing_campaign_id`
    - `parent_marketing_campaign_id`
- Governance:
  - promoted campaign revision workflow into semantic admin actions:
    - `clone`
    - `compare`
    - `submit-for-approval`
    - `approve`
    - `reject`
  - kept `publish / schedule / retire` as rollout actions, but gated rollout behind approval for governed revisions
- Audit:
  - extended campaign lifecycle audit with revision governance evidence:
    - `source_marketing_campaign_id`
    - `previous_approval_state / resulting_approval_state`
    - `previous_revision / resulting_revision`
- Contract:
  - added admin APIs:
    - `POST /admin/marketing/campaigns/{marketing_campaign_id}/clone`
    - `POST /admin/marketing/campaigns/{marketing_campaign_id}/compare`
    - `POST /admin/marketing/campaigns/{marketing_campaign_id}/submit-for-approval`
    - `POST /admin/marketing/campaigns/{marketing_campaign_id}/approve`
    - `POST /admin/marketing/campaigns/{marketing_campaign_id}/reject`
  - synchronized OpenAPI, admin TS types, and admin API client

## Verification

- RED:
  - new campaign revision-governance route test failed with `404`
  - OpenAPI inventory test failed because new campaign governance paths were absent
  - admin marketing API surface test failed because campaign revision-governance types and methods were absent
- GREEN:
  - `cargo test -p sdkwork-api-interface-admin --test marketing_coupon_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route -- --nocapture`
  - `node ./apps/sdkwork-router-admin/tests/admin-marketing-api-surface.test.mjs`

## Result

- campaign is no longer only a raw `create + status + publish` object; it now supports governed revision branching and approval semantics
- clone produces a new governed draft revision with lineage and revision evidence
- compare exposes structured field-level change review
- campaign `publish / schedule` no longer bypass approval on governed revisions

## Exit

- Step result: `conditional-go`
- Reason:
  - campaign revision governance is closed
  - `S03` still has remaining portal/public/account convergence work outside admin campaign governance

