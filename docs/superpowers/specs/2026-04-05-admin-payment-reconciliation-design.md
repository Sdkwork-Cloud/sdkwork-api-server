# Admin Payment Reconciliation Visibility Design

## Scope

This tranche exposes payment reconciliation evidence to operators through the admin API.

It focuses on the reconciliation records already persisted by the payment subsystem,
including the new refund provider conflict evidence.

## Problem

The payment subsystem now persists reconciliation lines for refund provider conflicts, but
those records are still effectively hidden unless someone reads the database directly.

That is not commercially acceptable for operating a payment system:

- suspicious refund replays must be visible to operators
- payment exceptions must be queryable without database access
- monitoring and incident response need an API surface to build on

## Design

### 1. Add an admin payment reconciliation list endpoint

Expose:

- `GET /admin/payments/reconciliation-lines`

Response type:

- `Vec<ReconciliationMatchSummaryRecord>`

### 2. Add store support for listing all reconciliation lines

The current store API only lists reconciliation lines by batch id. That is not sufficient
for operator views because operators do not know batch ids in advance.

Add a store method to list all reconciliation lines ordered by:

1. `created_at_ms DESC`
2. `reconciliation_line_id`

### 3. Keep this tranche read-only

This API is intentionally inspection-only:

- no mutation endpoints
- no filtering yet
- no pagination yet

The immediate goal is visibility for low-volume payment exceptions while keeping the code
change narrow.

## Testing strategy

Add an admin API regression test that:

1. creates a captured payment
2. creates a refund
3. finalizes it successfully
4. rewinds the refund to `processing`
5. replays success with a different provider refund id
6. calls `GET /admin/payments/reconciliation-lines`
7. asserts that one `mismatch_reference` reconciliation line is returned

## Out of scope

- filtering by payment order, refund order, or date range
- pagination and cursoring
- portal visibility
- webhook or alert fan-out
