# Refund Request Reuse Design

## Scope

This tranche hardens refund request creation against duplicate submissions.

It focuses on the request side of the refund loop:

- repeated identical refund requests from the same actor
- non-terminal refund orders that are already reserving refundable balance
- portal and internal flows that may retry or double-submit

## Problem

`request_payment_order_refund(...)` currently always creates a new refund order after
checking the remaining refundable amount.

That leaves a practical safety gap:

- a buyer or operator can submit the same partial refund twice
- both requests create independent non-terminal refund orders
- the second request consumes additional refundable headroom
- downstream approval and settlement flows now have to distinguish intent from accidental
  duplicate submission

Commercial payment systems should suppress exact duplicate refund requests while still
allowing intentionally different partial refunds.

## Design

### 1. Reuse exact-match non-terminal refund requests

Before computing remaining refundable amount for a new request, scan existing refund orders
 for the payment order and reuse the latest matching non-terminal refund order when all of
 the following match:

- `payment_order_id`
- `refund_reason_code`
- `requested_by_type`
- `requested_by_id`
- `requested_amount_minor`
- `refund_status` in:
  - `requested`
  - `awaiting_approval`
  - `approved`
  - `processing`

This suppresses duplicate clicks and idempotent client retries.

### 2. Keep distinct partial refunds possible

Do not reuse when any of these differ:

- refund amount
- reason code
- requester identity
- refund order is terminal (`succeeded`, `partially_succeeded`, `failed`, `canceled`)

This preserves valid business flows such as staged partial refunds.

### 3. Preserve payment-order pending status

If an existing reusable refund order is returned but the payment order refund status is not
`pending`, repair the payment order to `pending` before returning the reused refund order.

This keeps the order center projection consistent even when replay happens after unrelated
status drift.

## Testing strategy

Add regression coverage for:

1. direct payment-app duplicate partial refund requests returning the same refund order id
2. portal duplicate refund submissions returning the same refund order id and leaving only
   one refund order record persisted

## Out of scope

- cross-process race elimination with compare-and-swap or database locking
- idempotency-key headers for external clients
- admin refund approval workflows
