# Payment Over-Capture Governance Design

## Scope

This tranche hardens the payment kernel for settlement callbacks whose provider-reported captured
money exceeds the local payable amount.

The goal is:

- keep local accepted capture and refund ceilings bounded by `payable_minor`
- allow orders to reach a safe `captured` state even when the provider over-collected
- preserve single fulfillment and single account grant behavior
- record the provider excess through reconciliation evidence for operator follow-up

This builds directly on the earlier partial-capture and multi-capture slices.

## Problem

After the multi-capture aggregation tranche, the remaining commercial safety gap is over-capture.

Examples:

1. first capture overpay:
   - payable amount: `4000`
   - provider reports capture `4500`
2. replayed same-transaction overpay:
   - existing accepted capture: `1000`
   - provider later corrects the same transaction to `5000`
3. incremental capture crosses threshold with excess:
   - accepted captures: `1000 + 1500`
   - remaining payable: `1500`
   - provider reports next capture `1800`

Without explicit governance, local `captured_amount_minor` or refund ceilings can drift beyond the
commercially safe amount that should be recognized for the order.

That creates three risks:

- refunds can exceed what the order should ever refund locally
- account grants can be justified by more money than the order price
- operators lose a clean audit trail separating accepted capture from provider excess

## Approaches Considered

### Approach A: Accept the full provider amount locally

Pros:

- local transaction table mirrors the provider amount exactly

Cons:

- refund ceilings become unsafe unless a second excess-liability subsystem is added
- order accounting and benefit grants can overstate recognized revenue
- too much scope for this slice

### Approach B: Reject the entire capture whenever it exceeds payable

Pros:

- very simple safety rule

Cons:

- leaves orders locally unpaid even when the provider already collected enough to cover the order
- breaks fulfillment continuity and creates manual repair work

### Approach C: Accept only the locally safe amount and audit the excess

Pros:

- keeps order state, refund ceilings, and fulfillment bounded by payable
- still lets a fully covered order become `captured`
- preserves provider excess evidence for reconciliation and later operator handling

Cons:

- local sale amount can differ from provider-reported amount, so reconciliation evidence becomes
  mandatory

Recommended: Approach C.

## Design

### 1. Local accepted capture is capped

For every settled callback, compute the maximum locally acceptable amount:

- first sale or distinct sale transaction:
  - maximum accepted amount = remaining payable
- replay of the same sale transaction:
  - maximum accepted amount = payable minus accepted amounts from all other sale rows

Local accepted sale amount becomes:

- `min(provider_reported_amount, maximum_accepted_amount)`

This means:

- local aggregate capture never exceeds `payable_minor`
- local refund ceilings never exceed `payable_minor`

### 2. Over-capture still completes the order when enough money was collected

If the provider-reported amount is above the remaining payable, but the locally accepted amount
still reaches the payable threshold:

- payment order becomes `captured`
- fulfillment occurs once
- account grant occurs once
- `captured_amount_minor` is capped at `payable_minor`

The local system recognizes only the safe portion needed to satisfy the order.

### 3. Record excess as reconciliation evidence

Whenever provider-reported capture is larger than the local accepted capture:

- insert or update a reconciliation summary row
- use `match_status = mismatch_amount`
- keep provider amount = full provider-reported amount
- keep local amount = accepted local amount
- attach a reason code dedicated to capture capping

This creates an operator-visible paper trail that the provider charged more than the order could
accept locally.

### 4. Preserve existing conflict behavior where this slice should not expand scope

This slice does not introduce:

- automatic excess refund initiation
- separate excess-liability accounting
- dispute or chargeback handling
- operator UI workflows beyond the existing reconciliation surfaces

It only guarantees that the core order/payment state remains safe.

### 5. Refund ceiling stays tied to accepted local capture

Refund calculations continue to use local accepted capture, not provider excess:

- refund ceiling = accepted local aggregate capture
- remaining refundable amount = accepted local aggregate capture minus reserved/completed refunds

This prevents a provider overcharge from silently inflating portal or admin refund capabilities.

## Testing Strategy

Add regression coverage for:

1. app first-capture overpay:
   - provider reports `4500` on payable `4000`
   - payment order becomes `captured`
   - local `captured_amount_minor = 4000`
   - sale row local amount = `4000`
   - mismatch reconciliation line records provider `4500` vs local `4000`
2. app same-transaction replay overpay:
   - first callback `1000`
   - replay same provider transaction with `5000`
   - local accepted amount is capped to `4000`
   - no duplicate fulfillment
   - mismatch reconciliation evidence is present
3. portal or HTTP projection:
   - order center and callback responses expose capped local capture, not provider excess
   - refundable amount remains capped at `4000`

## Out Of Scope

- automatic excess refund workflow against Stripe / Alipay / WeChat Pay
- separate excess settlement journal entries
- dispute, chargeback, or reserve accounting
- operator remediation UI beyond current reconciliation listing
