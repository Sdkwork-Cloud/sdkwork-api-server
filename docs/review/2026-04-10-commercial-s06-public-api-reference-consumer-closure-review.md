# 2026-04-10 Commercial S06 Public API Reference Consumer Closure Review

## Scope

- Architecture reference: `166`
- Step reference: `108`
- Loop focus: close the remaining public consumer doc gap after runtime / OpenAPI convergence

## Findings

### P1 - gateway public consumer docs still did not publish the explicit market / marketing / commercial family contract

- the portal API reference center already mapped the live OpenAPI surface, but `docs/api-reference/gateway-api.md` still documented only OpenAI-compatible families
- impact:
  - downstream consumers could not read the public commercialization contract from the primary gateway API reference

### P1 - benefit-lot traversal and coupon-to-account-arrival evidence were still missing from the public doc surface

- `after_lot_id`, `next_after_lot_id`, and `scope_order_id` were runtime truth but not documented in the main gateway API reference
- impact:
  - pagination and account-arrival evidence remained implementation knowledge instead of productized public contract

## Fix Closure

- added explicit `market / marketing / commercial` route-family rows to `docs/api-reference/gateway-api.md`
- documented `after_lot_id`, `next_after_lot_id`, and `scope_order_id`
- kept the portal API reference center aligned with the same public workflow framing

## Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-api-reference-center.test.mjs`

## Residual Risks

- downstream SDK / client regeneration against the updated gateway schema has not been replayed in this loop

## Exit

- Step result: `conditional-go`
- Reason:
  - consumer-facing route-family and pagination documentation is now closed
  - schema-consumer regeneration remains open
