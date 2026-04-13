# Payment Partial Capture Safety Design

## Scope

This tranche closes the highest remaining payment safety gap after the authorization lifecycle
work: verified settlement callbacks that capture less than the order payable amount must not be
treated as full successful payment.

The scope is intentionally narrow and commercial-grade:

- represent partial capture on a single payment order
- persist captured money separately from payable money
- block fulfillment, quota grants, and full refund ceilings until full capture happens
- keep callback replay idempotent when the same provider transaction later advances from partial
  capture to full capture

This tranche does not implement multi-capture accounting with multiple distinct sale rows. It
creates the safe foundation first.

## Problem

Today, any verified `settled` callback is effectively treated as "full payment completed" because
the system still assumes:

- `payment_status = captured`
- full entitlement fulfillment is allowed
- recharge quota/account grants can be issued
- refund capacity is derived from `payable_minor`

That is unsafe when a provider reports a partial capture. A concrete example:

- order payable amount: `4000`
- provider capture callback amount: `1000`

Current behavior risks:

- the commerce order being fulfilled as if `4000` was collected
- the user receiving full recharge quota or entitlement activation
- refunds being allowed up to `4000` even though only `1000` was actually captured
- audit trails overstating settled revenue

For a commercial order center and payment platform, this is a hard correctness bug.

## Design

### 1. Represent partial capture explicitly

Add `PartiallyCaptured` to `PaymentOrderStatus`.

Add `captured_amount_minor` to `PaymentOrderRecord`.

Meaning:

- `payable_minor`: what the order expects to collect
- `captured_amount_minor`: what has actually been captured and is eligible for refund/revenue
  recognition logic in this tranche

Rules:

- default `captured_amount_minor = 0`
- authorization does not raise it
- partial settlement sets it to the settled amount
- later settlement upgrades it monotonically

### 2. Split settlement into partial and full capture paths

On a verified `settled` callback:

- if captured amount is `0`, ignore it for progression
- if captured amount is less than `payable_minor`:
  - set payment order status to `partially_captured`
  - set fulfillment status to `partial_capture_pending_review`
  - persist or update the canonical sale transaction to the partial amount
  - do not fulfill commerce order
  - do not issue account quota / grant side effects
- if captured amount is at least `payable_minor`:
  - set payment order status to `captured`
  - set fulfillment status to `captured_pending_fulfillment`
  - update `captured_amount_minor`
  - persist/update the canonical sale transaction
  - run fulfillment exactly once

### 3. Make replay progression monotonic

The canonical sale transaction remains one row per payment order in this tranche.

If the same provider transaction id later arrives with a larger captured amount:

- update the canonical sale transaction amount upward
- update `captured_amount_minor` upward
- if the order crosses from partial to full capture, then perform fulfillment

If a later callback reports a smaller amount than already recorded:

- keep the larger local captured amount
- do not downgrade order state

If a different provider transaction id appears for the same order, keep the existing provider
conflict reconciliation behavior.

### 4. Refund eligibility must use captured money, not payable money

Refund support expands to include `partially_captured`, but the allowed ceiling changes:

- refundable ceiling = `captured_amount_minor`
- remaining refundable = `captured_amount_minor - reserved_or_completed_refunds`

This affects:

- refund request validation
- portal/admin order-center refundable amount display
- derived refund status (`not_requested`, `pending`, `partially_refunded`, `refunded`)

That means a `1000` partial capture can refund at most `1000`, never the full order payable amount.

### 5. Preserve operational visibility

Portal and admin-facing payment/order history should expose:

- `payment_status = partially_captured`
- `captured_amount_minor`
- sale transaction amount equal to the actually captured amount
- `fulfillment_status = partial_capture_pending_review`
- `refundable_amount_minor` based on captured money

This keeps operators from mistaking a partial capture for full settlement.

## Testing strategy

Add regression coverage for:

1. partial capture callback:
   - payment order becomes `partially_captured`
   - `captured_amount_minor` equals the callback amount
   - fulfillment does not happen
   - no account grant side effects occur
   - canonical sale transaction amount equals the partial capture
2. refund ceiling:
   - refund requests above `captured_amount_minor` fail
   - refund requests within `captured_amount_minor` succeed
3. portal order center:
   - shows `partially_captured`
   - shows `partial_capture_pending_review`
   - exposes `refundable_amount_minor` from captured money
4. capture upgrade replay:
   - partial capture followed by full capture on the same provider transaction upgrades the order
   - fulfillment runs once on the upgrade
   - canonical sale transaction amount upgrades instead of duplicating
5. storage schema:
   - SQLite and Postgres persist `captured_amount_minor`

## Out of scope

- multiple distinct capture transactions for one payment order
- provider-driven incremental authorization adjustments
- partial fulfillment of digital entitlements
- manual operator tooling for approving partial capture exceptions
