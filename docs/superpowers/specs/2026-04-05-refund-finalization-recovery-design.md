# Refund Finalization Recovery Design

## Scope

This tranche hardens successful refund finalization for crash and replay recovery.

It focuses on one specific commercial risk window:

- a refund order has already emitted some or all downstream side effects
- the refund order record is still `processing`
- the success path is replayed again later

The target is deterministic convergence without duplicate financial evidence.

## Problem

The current refund success path already guards the two mutable balance reversals:

- account grant reversal is deduped by `refund_order_id`
- quota reversal is deduped by `refund_order_id`

Finance journal entry and lines also use deterministic ids, so replays upsert into the
same records.

The remaining weak edge is the refund payment transaction id. It is currently derived
from `payment_order_id + provider_refund_id`, which means a replay of the same
`refund_order` with a different provider refund id can create a second refund
transaction row while all other refund evidence remains singular.

That breaks payment ledger integrity for the same commercial refund.

## Design

### 1. Anchor refund transaction identity to `refund_order_id`

Treat the commercial refund order as the canonical local identity for the refund
transaction.

Change refund transaction ids from:

- `payment_transaction_<payment_order_id>_<provider_refund_id>`

to:

- `payment_transaction_refund_<refund_order_id>`

This guarantees one local refund transaction row per refund order regardless of replay
parameters.

### 2. Preserve the first persisted refund transaction on replay

When replaying `finalize_refund_order_success(...)`:

- list payment transactions for the order
- if the canonical refund transaction row already exists, reuse it
- do not overwrite its provider refund id or timestamps during replay

This keeps reconciliation evidence stable after the first successful persistence.

### 3. Keep existing side-effect idempotency strategy

No new storage step table is required for:

- refund payment transaction
- finance journal entry
- finance journal lines

The combination of deterministic ids and replay-aware reuse is sufficient.

### 4. Regression coverage

Add a recovery-focused regression test that simulates:

1. successful refund finalization
2. refund order status manually rewound to `processing`
3. replay with a different `provider_refund_id`

Expected outcome:

- exactly one refund transaction remains
- the original provider refund id is preserved
- journal entry count remains one
- journal line count remains two
- refund order converges back to `succeeded`

## Out of scope

- legacy migration of previously persisted refund transactions created with the old id
  format
- provider-side refund creation APIs
- provider reconciliation backfill against historical duplicate rows
