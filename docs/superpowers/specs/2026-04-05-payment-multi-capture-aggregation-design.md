# Payment Multi-Capture Aggregation Design

## Scope

This tranche upgrades the payment kernel from "one sale transaction per payment order" to a
commercially safer aggregation model that can represent multiple distinct settlement captures for
the same order.

The goal is:

- allow several provider capture transactions to accumulate against one payment order
- keep `captured_amount_minor` equal to the aggregate locally accepted captured money
- delay fulfillment and account grants until aggregate captured money reaches the payable amount
- keep refund ceilings tied to the aggregate captured amount

This builds directly on the earlier partial-capture safety slice.

## Problem

The current implementation only safely supports:

- one authorization transaction
- one canonical sale transaction per payment order
- one refund transaction per refund order

That means repeated settlement callbacks with a new provider transaction id are treated as a
reference conflict even when they are legitimate incremental captures. For providers or enterprise
workflows that legally capture in multiple steps, the current behavior is still below a
commercial-grade payment platform.

Concrete example:

- payable amount: `4000`
- capture `1000` on provider transaction `cap_1`
- later capture `1500` on provider transaction `cap_2`

Today, the second capture is not modeled as money actually collected for the order. That leaves the
order center, refund eligibility, and audit trail incomplete.

## Design

### 1. One sale row per provider capture transaction

For settlement callbacks:

- same provider transaction id: update the existing sale row monotonically
- different provider transaction id: create a second sale row if the amount still fits within the
  remaining payable amount

This gives the order center a truthful transaction history:

- `cap_1` -> `1000`
- `cap_2` -> `1500`

instead of collapsing all captures into one row or misclassifying them as conflicts.

### 2. Aggregate captured money from sale rows

`captured_amount_minor` becomes the sum of accepted sale transactions for the payment order.

Rules:

- same-transaction replay can only raise a sale row amount, never lower it
- aggregate captured money is recomputed from accepted sale rows
- `payment_status = partially_captured` while aggregate captured is below `payable_minor`
- `payment_status = captured` once aggregate captured reaches `payable_minor`

### 3. Fulfill exactly once on threshold crossing

Commerce fulfillment, account grants, and related side effects must still happen exactly once.

Rules:

- partial captures do not fulfill
- the first callback that brings aggregate captured money to the payable threshold triggers
  fulfillment
- later replays or duplicate callbacks do not re-fulfill

### 4. Keep refund ceilings aggregated

Refund support must continue to use accepted captured money, but now across multiple sale rows:

- refund ceiling = aggregate accepted `captured_amount_minor`
- remaining refundable amount = aggregate captured minus reserved/completed refunds

That means two accepted captures of `1000` and `1500` allow at most `2500` in refunds until the
final capture arrives.

### 5. Preserve safety when extra captures exceed payable

This tranche does not yet implement over-capture acceptance or a special over-capture financial
workflow.

For now:

- multi-capture is allowed only while the new capture amount fits within the remaining payable
  amount
- a distinct capture that would push the order beyond payable continues to be blocked locally and
  audited through reconciliation evidence instead of creating an extra sale row

This keeps the slice safe and bounded.

## Testing strategy

Add regression coverage for:

1. app multi-capture aggregation:
   - `cap_1 = 1000`, `cap_2 = 1500`
   - payment order remains `partially_captured`
   - `captured_amount_minor = 2500`
   - two sale transactions exist
   - no fulfillment/account side effects
2. app threshold crossing:
   - add `cap_3 = 1500`
   - payment order becomes `captured`
   - fulfillment and account grants happen once
   - three sale transactions exist
3. refund ceiling:
   - refund requests above aggregate captured money fail
   - refund requests at or below the aggregate captured money succeed
4. HTTP route:
   - repeated settlement callbacks with distinct provider transaction ids accumulate accepted sale
     rows and aggregate captured money
5. portal order center:
   - shows multiple sale transactions
   - shows aggregated `captured_amount_minor`
   - shows aggregated refundable amount

## Out of scope

- over-capture financial acceptance workflow
- capture reversal / partial void workflow
- per-capture refund attribution
- dispute / chargeback modeling
- operator UI for multi-capture settlement review
