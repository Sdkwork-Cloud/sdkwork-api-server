# Refund Provider Conflict Audit Design

## Scope

This tranche hardens refund replay handling when the same local `refund_order` is finalized
more than once with different provider refund ids.

The goal is:

- keep the local refund close loop convergent and idempotent
- preserve the first accepted refund transaction evidence
- record later provider id conflicts as explicit reconciliation evidence

## Problem

The previous tranche fixed duplicate local refund transactions by anchoring the refund
transaction id to `refund_order_id`.

That closed the duplicate-row problem, but it still leaves an observability gap:

- replay of the same `refund_order` with a different `provider_refund_id` no longer creates
  a second transaction row
- the system silently keeps the first provider refund id
- operators lose evidence that conflicting provider references were observed

For commercial payment systems this is too weak. A mismatched provider refund reference
must be auditable for reconciliation and incident response.

## Design

### 1. Reconciliation evidence for refund provider conflicts

When `finalize_refund_order_success(...)` sees that the canonical local refund transaction
already exists and its `provider_transaction_id` differs from the newly supplied
`provider_refund_id`, persist a reconciliation line.

The reconciliation record will include:

- tenant and organization ids from the payment order
- payment order id
- refund order id
- conflicting provider refund id
- local refund amount
- provider amount from the replay input
- reason code: `refund_provider_transaction_conflict`
- match status: `mismatch_reference`

### 2. Deterministic conflict ids

Use deterministic ids so conflict recording is replay-safe:

- reconciliation batch id: one batch per refund order conflict stream
- reconciliation line id: one line per `refund_order_id + conflicting_provider_refund_id`

This allows repeated delivery of the same conflicting provider id to upsert cleanly without
duplicating operator evidence.

### 3. Preserve convergent refund close loop

Conflict recording must not break refund completion.

If the canonical refund transaction already exists:

- return the existing local refund transaction
- do not overwrite the first persisted provider refund id
- record the conflict when the new provider refund id differs

This keeps financial state stable while still surfacing suspicious input.

## Testing strategy

Add payment-app regression coverage that:

1. finalizes a refund successfully
2. rewinds the refund order back to `processing`
3. replays success with a different provider refund id
4. asserts:
   - still only one refund transaction exists
   - the original provider refund id is preserved
   - one reconciliation record exists for the conflicting provider id
   - the reconciliation reason code is `refund_provider_transaction_conflict`
   - the match status is `mismatch_reference`

## Out of scope

- admin API exposure for reconciliation lines
- automated alert fan-out to external monitoring systems
- provider-side refund verification queries
