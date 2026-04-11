# 2026-04-10 Commercial S03 Admin Marketing Coupon Template Revision Governance Review

## Scope

- Architecture reference: architecture doc `166`
- Step reference: step doc `105`
- Loop focus: close canonical `coupon template` revision, approval, compare, and clone governance

## Findings

### P1 - template had rollout lifecycle but no governed revision branch

- admin exposed `publish / schedule / retire`, but lacked `clone / compare / submit-for-approval / approve / reject`
- impact:
  - template changes could not branch cleanly into a reviewable draft revision
  - operators had to treat lifecycle mutation as both authoring and rollout control

### P1 - template rollout had no approval gate for governed revisions

- template publish/schedule was still status-driven and not approval-aware
- impact:
  - draft revisions could move toward rollout without a first-class approval checkpoint
  - marketing control stayed below commercial-grade governance expectations

### P2 - template audit lacked revision and approval evidence

- lifecycle audit only tracked status transitions
- impact:
  - clone lineage, approval decisions, and revision increments were not durable evidence
  - admin review tooling could not reconstruct why one revision superseded another

## Fix Closure

- added template governance fields:
  - `approval_state`
  - `revision`
  - `root_coupon_template_id`
  - `parent_coupon_template_id`
- added semantic admin routes for:
  - `clone`
  - `compare`
  - `submit-for-approval`
  - `approve`
  - `reject`
- extended lifecycle audit with:
  - `source_coupon_template_id`
  - `previous_approval_state / resulting_approval_state`
  - `previous_revision / resulting_revision`
- enforced approval-aware rollout:
  - governed draft revision must be approved before `publish / schedule`
- synchronized backend routes, OpenAPI, admin TS types, and admin API client

## Verification

- RED:
  - template revision-governance route test failed with `404`
  - OpenAPI inventory test failed because new template governance paths were absent
  - admin marketing API surface failed because revision-governance types and methods were missing
- GREEN:
  - `cargo test -p sdkwork-api-interface-admin --test marketing_coupon_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-admin --test openapi_route -- --nocapture`
  - `node ./apps/sdkwork-router-admin/tests/admin-marketing-api-surface.test.mjs`

## Residual Risks

- campaign still lacks the same `revision / approval / compare / clone` governance layer
- raw compatibility create/status flows still exist; they are no longer the preferred governance path
- broader `S03` portal/public/account convergence is still open

## Exit

- Step result: `conditional-go`
- Reason:
  - template revision governance is now commercial-grade
  - full `S03` commercialization is still not complete
