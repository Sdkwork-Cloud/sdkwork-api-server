# Refund Close-Loop Design

## Scope

This tranche closes the canonical refund loop for paid recharge orders:

- create a refund order against a captured payment order
- prevent over-refunding and unsupported refund targets
- finalize a successful refund idempotently
- reverse recharge quota and account grant balances
- persist refund transaction and finance journal evidence
- update payment order and refund order statuses consistently

Out of scope for this tranche:

- provider-specific refund request adapters for Stripe, WeChat Pay, or Alipay
- refund approval workflows
- subscription membership rollback and proration
- coupon restoration policy

## Problem

The workspace now settles payment callbacks into commerce fulfillment and account grants, but refund handling is still only a domain/storage skeleton. That leaves a commercial gap:

- payment orders can be captured without a canonical refund request flow
- refund orders are not orchestrated
- account history does not show negative refund reversals
- quota remains inflated after a recharge refund
- finance journals do not capture refund payout evidence

This makes the order center incomplete for real money movement.

## Design

### 1. Refund orchestration in payment app

Add two application operations:

- `request_payment_order_refund(...)`
- `finalize_refund_order_success(...)`

`request_payment_order_refund(...)` will:

- load the payment order
- require `payment_status = captured`
- require a supported commerce order target kind (`recharge_pack` or `custom_recharge`)
- calculate remaining refundable amount from prior refund orders
- create a refund order with approved amount equal to requested amount
- mark the payment order refund status as `pending`

`finalize_refund_order_success(...)` will:

- load refund order, payment order, and commerce order
- no-op safely if the refund order is already terminal success
- persist a refund payment transaction using deterministic ids
- reverse recharge quota through a transactional refund step
- reverse account grant through a transactional refund step
- persist finance journal entry and lines
- update the refund order to `succeeded`
- recalculate payment order refund status to `partially_refunded` or `refunded`

### 2. Recharge-only refund policy

This tranche intentionally supports only recharge orders:

- `recharge_pack`
- `custom_recharge`

These targets have a direct quantity model and can be reversed proportionally. `subscription_plan` remains unsupported until membership rollback semantics are designed.

### 3. Idempotent reversal steps

Refund success replay must not subtract quota or account balances twice. Introduce refund-step transactional guards in storage:

- quota reversal keyed by `refund_order_id`
- account reversal keyed by `refund_order_id`

Each guarded storage method inserts the refund processing step and applies the mutable side effect inside the same database transaction.

### 4. Proportional quantity reversal

For supported recharge refunds, reverse quantity as:

`(refunded_amount_minor / payable_minor) * granted_quantity`

Rules:

- reject refund if payable amount is zero
- reject refund if the original grant lot no longer has enough unconsumed quantity for the requested reversal
- set lot status to `exhausted` when the remaining quantity reaches zero

### 5. Financial evidence

Persist finance journal evidence for each successful refund:

- entry code: `refund_payout`
- source kind: `refund_order`
- source id: refund order id
- line 1: debit `customer_prepaid_liability`
- line 2: credit `payment_refund_clearing`

These records are idempotent by deterministic ids.

## Testing strategy

Add refund TDD coverage for:

- full recharge refund creates refund order and marks payment order pending refund
- successful refund is idempotent and reverses quota/account history once
- partial refund marks payment order as partially refunded and reverses proportional quantity
- unsupported subscription refund is rejected

## Risks deliberately left for next tranche

- subscription downgrade and membership cancellation/refund policy
- coupon restoration after refund
- provider refund callbacks and asynchronous provider reconciliation
