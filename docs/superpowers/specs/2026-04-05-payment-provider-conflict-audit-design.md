# Payment Provider Conflict Audit Design

## Scope

This tranche hardens verified settlement callback replay handling when the same local
`payment_order` is processed more than once with different provider transaction ids.

The goal is:

- keep local payment settlement convergent and idempotent
- preserve the first accepted sale transaction evidence
- record later provider transaction conflicts as explicit reconciliation evidence

## Problem

The current sale transaction identity is still derived from
`payment_order_id + provider_transaction_id`.

That is adequate for exact duplicate callbacks, but it leaves a replay gap:

- a second verified settlement callback with a new dedupe key can still be processed
- if its `provider_transaction_id` differs, the system can create a second local sale
  transaction row
- operators lose a clear signal that conflicting provider references were observed for
  the same payment order

For commercial payment systems this is too weak. Sale-side provider reference conflicts
must be convergent locally and visible to reconciliation tooling.

## Design

### 1. Canonical sale transaction reuse

Verified settlement processing should reuse an existing local sale transaction for the
payment order instead of creating a second one when settlement is replayed.

For newly created records, introduce a canonical local sale transaction id anchored only
to `payment_order_id`.

To preserve upgrade compatibility, the reuse path must also detect legacy sale
transactions already written with the old
`payment_order_id + provider_transaction_id` identity scheme and treat them as the
canonical sale transaction for that order.

### 2. Reconciliation evidence for provider reference conflicts

When the canonical local sale transaction already exists and its
`provider_transaction_id` differs from the newly supplied provider transaction id,
persist a reconciliation line instead of inserting another sale transaction.

The reconciliation record will include:

- tenant and organization ids from the payment order
- payment order id
- conflicting provider transaction id
- local sale amount
- provider amount from the callback input
- reason code: `payment_provider_transaction_conflict`
- match status: `mismatch_reference`

### 3. Deterministic conflict ids

Use deterministic ids so conflict recording is replay-safe:

- reconciliation batch id: one batch per payment order conflict stream
- reconciliation line id: one line per `payment_order_id + conflicting_provider_transaction_id`

Repeated delivery of the same conflicting provider transaction id should upsert cleanly
without duplicating operator evidence.

### 4. Duplicate callback hydration fallback

Callback duplicate hydration currently resolves transactions by matching the callback's
provider transaction id to a stored payment transaction row.

After canonical sale anchoring, conflicting callback events may legitimately refer to a
provider transaction id that is not stored on the canonical sale transaction. Hydration
should therefore fall back to the canonical sale transaction for the payment order when
an exact provider id match is not present.

## Testing strategy

Add payment-app regression coverage that:

1. settles a payment successfully
2. replays a second verified settlement callback with a distinct dedupe key and a
   different provider transaction id
3. asserts:
   - still only one sale transaction exists
   - the original provider transaction id is preserved
   - one reconciliation record exists for the conflicting provider transaction id
   - the reconciliation reason code is `payment_provider_transaction_conflict`
   - the match status is `mismatch_reference`

## Out of scope

- automated provider-side capture verification queries
- asynchronous alert fan-out to external monitoring systems
- multi-capture or split-settlement modeling
