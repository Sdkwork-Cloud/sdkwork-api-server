# Admin Payment Order Dossier Design

## Scope

This tranche adds a single admin drilldown endpoint for one payment order so operators can inspect
the complete commercial evidence chain without joining multiple APIs by hand.

It covers:

- admin read access to one payment order dossier
- commerce order linkage
- payment attempts and sessions
- callback audit events
- payment and refund transactions
- refund orders
- reconciliation lines related to the order or its refunds
- account and account ledger evidence related to the order
- finance journal evidence related to the order refunds

It does not cover:

- new payment or refund write flows
- mutation of reconciliation state beyond the existing resolve endpoint
- CSV export
- portal-facing dossier inspection
- cross-order account timeline views

## Problem

The payment/order/refund/account closure has become materially stronger, but the operator
inspection surface is still fragmented:

- `/admin/payments/orders` gives only the top-level payment order rows
- `/admin/payments/refunds` gives refund rows, but not the surrounding attempt/session/callback
  chain
- `/admin/payments/reconciliation-lines` shows exceptions, but not the full local evidence for the
  affected order
- account ledger and finance journal records exist, but operators cannot pivot from a payment order
  to the commercial accounting evidence that was created from it

This is below commercial operating standards for support, finance, risk review, and incident
response. A production operator needs one deterministic payload that answers: what was sold, how it
was paid, what retried, what refunded, what was written to the account system, and what accounting
evidence exists.

## Recommended Approach

Add a new admin endpoint:

- `GET /admin/payments/orders/{payment_order_id}`

Implement the dossier assembly in `sdkwork-api-app-payment` as a read model over the existing
payment and account kernels, then expose it through the admin interface crate. This keeps the
commercial evidence stitching in the payment application layer instead of duplicating business
knowledge inside the HTTP adapter.

## Response Design

The response should include:

- `payment_order`
- `commerce_order`
- `payment_attempts`
- `payment_sessions`
- `payment_callback_events`
- `payment_transactions`
- `refund_orders`
- `reconciliation_lines`
- `account`
- `account_ledger_entries`
- `account_ledger_allocations`
- `finance_journal_entries`
- `finance_journal_lines`

### Filtering rules

- `payment_attempts` are loaded from `list_payment_attempt_records_for_order`
- `payment_sessions` are all sessions belonging to the returned attempts
- `payment_callback_events` are events whose `payment_order_id` matches the requested order or
  whose `payment_attempt_id` belongs to one of the returned attempts
- `payment_transactions` are loaded from `list_payment_transaction_records_for_order`
- `refund_orders` are loaded from `list_refund_order_records_for_payment_order`
- `reconciliation_lines` are relevant when either:
  - `payment_order_id` equals the requested order, or
  - `refund_order_id` belongs to one of the returned refunds
- `account` is the primary account for the order owner when present
- `account_ledger_entries` are only the ledger entries deterministically produced by the commerce
  order grant and its refund reversals, not the user’s full account history
- `account_ledger_allocations` are allocations attached to the returned ledger entries
- `finance_journal_entries` are finance entries whose source ids match the requested payment order
  or one of its refund orders
- `finance_journal_lines` are all lines belonging to the returned finance entries

### Ordering rules

- attempts: `attempt_no DESC`, then `created_at_ms DESC`, then `payment_attempt_id ASC`
- sessions: `created_at_ms DESC`, then `payment_session_id DESC`
- callback events: `received_at_ms DESC`, then `callback_event_id DESC`
- transactions: `occurred_at_ms DESC`, then `payment_transaction_id DESC`
- refunds: `created_at_ms DESC`, then `refund_order_id DESC`
- reconciliation lines: reuse the existing admin reconciliation ordering
- account ledger entries: `created_at_ms DESC`, then `ledger_entry_id DESC`
- account ledger allocations: `created_at_ms DESC`, then `ledger_allocation_id DESC`
- finance journal entries: `occurred_at_ms DESC`, then `finance_journal_entry_id DESC`
- finance journal lines: by parent finance entry order, then `line_no ASC`, then
  `finance_journal_line_id ASC`

## Error Handling

- missing payment order returns `404 Not Found`
- unsupported store backends keep the existing admin capability-unavailable behavior and surface
  `500`
- orders without an account still return `200` with `account: null` and empty account evidence
  arrays
- orders without finance journal entries still return `200` with empty finance arrays

## Design Notes

### Why the dossier belongs in `sdkwork-api-app-payment`

The read model needs payment/order/refund knowledge plus deterministic linkage to account ledger
and finance journal evidence. Keeping that stitching inside the payment application layer makes the
admin interface thinner and keeps future consumers free to reuse the same dossier loader.

### Why account evidence is filtered narrowly

An operator drilling into one order needs the records caused by that order, not the user’s full
account history. Returning the full account history would create noise, hide causality, and make
large accounts harder to inspect under production load.

### Why finance evidence is source-id based

Refund journal entries already use canonical `source_kind` and `source_id`. Filtering by those
stable ids preserves determinism and avoids brittle timestamp-only correlation.

## Testing Strategy

Add admin integration coverage for:

1. a happy-path dossier with commerce order, attempts, sessions, callbacks, transactions, and
   refunds
2. a refund-completed dossier that also exposes account ledger and finance journal evidence
3. a dossier with reconciliation lines caused by refund provider conflict replay
4. missing payment order returns `404`

Re-run the existing admin and payment regression suites after the new endpoint lands.
