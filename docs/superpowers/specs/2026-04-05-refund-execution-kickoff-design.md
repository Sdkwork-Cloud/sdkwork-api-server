# Refund Execution Kickoff Design

## Scope

This tranche closes the operational gap between refund approval and refund completion.

It covers:

- an explicit admin action to start approved refund execution
- a stricter portal refund lifecycle:
  - `awaiting_approval`
  - `approved`
  - `processing`
  - `succeeded` or `partially_succeeded`
- admin refund queue filtering by `refund_status`
- refund success finalization rejecting portal-governed refunds that are approved but not started

It does not cover:

- async refund job tables or outbox dispatch
- provider-specific refund API dispatch workers
- multi-stage approval chains
- chargeback and dispute management

## Problem

The current refund governance slice introduced `awaiting_approval`, but `approved` still acts as a
soft marker instead of a true operational boundary. A refund can be approved and then jump directly
into success finalization without any explicit "execution has started" transition.

That leaves three commercial-grade gaps:

- operators cannot distinguish approved-but-not-started refunds from in-flight refunds
- provider callbacks can finalize a refund that was approved but never dispatched
- admin refund queues cannot be filtered to isolate approval backlog versus execution backlog

## Recommended Approach

Add a narrow execution kickoff layer without introducing new tables:

- keep internal low-level refund compatibility:
  - legacy direct flows may still finalize from `requested`
- tighten the governed portal path:
  - `awaiting_approval -> approved -> processing -> succeeded`
- add:
  - `POST /admin/payments/refunds/{refund_order_id}/start`
  - `GET /admin/payments/refunds?refund_status=<status>`

This keeps the slice small, improves operator control immediately, and creates a clean seam for a
future async dispatch/outbox system.

## Behavior Design

### Admin start refund execution

Request:

- `POST /admin/payments/refunds/{refund_order_id}/start`

Body:

- `started_at_ms` required

Rules:

- allowed transitions:
  - `approved -> processing`
  - `processing -> processing` idempotent
- rejected states:
  - `requested`
  - `awaiting_approval`
  - `failed`
  - `canceled`
  - `partially_succeeded`
  - `succeeded`
- starting execution keeps the parent payment order in `refund_status = pending`
- the refund row updates `updated_at_ms` on the first successful start

### Refund success finalization

`finalize_refund_order_success` must reject:

- `awaiting_approval`
- `approved`
- `failed`
- `canceled`
- `partially_succeeded`

It may continue when the refund is:

- `requested` for existing low-level internal compatibility flows
- `processing`
- already `succeeded` for replay safety

This means portal-governed refunds require an explicit start action before provider success can
settle the refund.

### Admin refund queue filtering

Request:

- `GET /admin/payments/refunds`

Query:

- `refund_status` optional, exact enum value

Rules:

- absent `refund_status` returns the full refund queue
- when present, only rows with that exact `refund_status` are returned
- invalid enum values return `400 BAD_REQUEST`
- sorting remains newest-first by `created_at_ms`, then `refund_order_id`

This supports operational queue views such as:

- `awaiting_approval`
- `approved`
- `processing`

## API Surface

### New admin route

- `POST /admin/payments/refunds/{refund_order_id}/start`

Request body:

```json
{
  "started_at_ms": 1710603400
}
```

Response:

- canonical `RefundOrderRecord`

### Updated admin list route

- `GET /admin/payments/refunds?refund_status=processing`

Response:

- filtered `Vec<RefundOrderRecord>`

## Testing Strategy

Add coverage for:

1. approved refunds cannot be finalized before execution is started
2. admin can start an approved refund and receives `processing`
3. admin refund list filters by `refund_status`
4. started refunds can still complete through the existing success finalization path

Re-run focused payment/admin/portal regressions after implementation.
