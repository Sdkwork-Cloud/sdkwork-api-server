# Refund Approval Governance Design

## Scope

This tranche turns portal-initiated refunds into an approval-gated workflow instead of allowing
them to flow directly into refund settlement.

It covers:

- portal refund requests entering `awaiting_approval`
- admin approval of a refund request with an optional approved amount
- admin cancellation of a refund request before execution
- refund-success finalization rejecting refund requests that are still awaiting approval

It does not cover:

- separate refund-approval audit tables
- multi-step approval chains
- provider-side refund dispatch jobs
- automated risk scoring
- chargeback dispute handling

## Problem

The payment/refund/account loop is already idempotent and financially consistent, but it still has
an operational governance gap: customer-originated refund requests can move toward completion
without a true approval gate. The domain already contains `RefundOrderStatus::AwaitingApproval`,
but the status is not active in the current product flow.

That leaves commercial risk:

- customer refund requests are not clearly separated from approved refunds
- providers or operators can settle a refund request that has not yet been approved
- support teams lack a deterministic admin action to approve or cancel queued refund requests

## Recommended Approach

Activate approval gating only on the portal-originated refund path in this tranche:

- portal refund submission persists the refund order as `awaiting_approval`
- admin gains:
  - `POST /admin/payments/refunds/{refund_order_id}/approve`
  - `POST /admin/payments/refunds/{refund_order_id}/cancel`
- refund finalization succeeds only when the refund order is already `approved`, `processing`, or
  already terminal-success for replay handling

This keeps the change small, avoids breaking internal low-level refund helpers that are still used
by backend tests and seeds, and upgrades the real customer-facing path first.

## Behavior Design

### Portal refund request

When the portal user submits a refund request:

- create or reuse the canonical refund order through the existing refund request helper
- immediately transition the stored refund order to:
  - `refund_status = awaiting_approval`
  - `approved_amount_minor = null`
- keep the parent payment order in `refund_status = pending`

### Admin approve

Request:

- `POST /admin/payments/refunds/{refund_order_id}/approve`

Body:

- `approved_amount_minor` optional
- `approved_at_ms` required

Rules:

- only `requested` or `awaiting_approval` can be newly approved
- `approved_amount_minor` defaults to `requested_amount_minor`
- approved amount must be positive and must not exceed the requested amount
- already `approved` returns the updated canonical row idempotently when the approved amount is the
  same
- `processing`, `partially_succeeded`, `succeeded`, `failed`, and `canceled` are not approvable

### Admin cancel

Request:

- `POST /admin/payments/refunds/{refund_order_id}/cancel`

Body:

- `canceled_at_ms` required

Rules:

- only pre-execution refund requests can be canceled:
  - `requested`
  - `awaiting_approval`
  - `approved`
- `canceled` is idempotent
- canceling recomputes the parent payment order refund summary from remaining refund rows

### Refund success finalization

`finalize_refund_order_success` must reject:

- `awaiting_approval`
- `failed`
- `canceled`
- `partially_succeeded`

It may continue when the refund is:

- `requested` for existing low-level internal refund flows that have not yet been migrated onto
  the approval gate
- `approved`
- `processing`
- already `succeeded` for replay safety

## Testing Strategy

Add coverage for:

1. portal refund request returning `awaiting_approval`
2. repeated portal refund submission reusing the same awaiting-approval refund order
3. admin approval and cancellation routes
4. refund success finalization failing for an awaiting-approval refund order

Re-run payment, admin, and portal regressions after implementation.
