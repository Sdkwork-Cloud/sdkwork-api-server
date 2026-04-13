# Payment Reconciliation Point Lookup Design

## Scope

This tranche hardens the reconciliation resolution path so admin mutation flows do not degrade to
full-list scans as anomaly volume grows.

It keeps the external admin API unchanged and focuses on the storage and service path used by:

- `POST /admin/payments/reconciliation-lines/{reconciliation_line_id}/resolve`

## Problem

The current resolution path loads all reconciliation lines and then finds the target line in
memory.

That is acceptable for tiny datasets but weak for commercial payment operations:

- resolve latency grows with total reconciliation history
- operators pay the cost of historical audit volume even for single-line mutations
- the implementation ignores the fact that reconciliation line id is already a primary key

This is unnecessary work in the hottest mutation path that follows operator triage.

## Approaches Considered

### Approach A: Keep scanning the full reconciliation list

Pros:

- no new store method

Cons:

- scales poorly
- couples point reads to list behavior
- leaves an avoidable performance footgun in place

### Approach B: Add a point lookup to the payment kernel store

Pros:

- matches the existing primary-key data model
- keeps mutation latency stable as history grows
- reuses existing row decoders and persistence types

Cons:

- requires touching store trait plus both sqlite/postgres implementations

Recommended: Approach B.

## Design

### 1. Add a point-read method to `PaymentKernelStore`

Introduce:

- `find_reconciliation_match_summary_record(&self, reconciliation_line_id: &str) -> Result<Option<ReconciliationMatchSummaryRecord>>`

This becomes the canonical point-lookup API for reconciliation lines.

### 2. Implement direct primary-key reads in sqlite and postgres

Each store performs:

- a single `SELECT ... FROM ai_payment_reconciliation_line WHERE reconciliation_line_id = ?/$1`

The query reuses the existing reconciliation row decode logic so the data contract stays
consistent with list endpoints.

### 3. Update admin resolve to use the new point lookup

The admin resolution flow changes from:

- list all lines
- in-memory filter by id

To:

- fetch one line by id
- return `404` when absent
- resolve and upsert when present

### 4. Leave queue listing behavior untouched

This slice does not modify:

- reconciliation list filtering
- queue ordering
- schema or indexes

The existing primary key is sufficient for direct lookup.

## Testing Strategy

Add regression coverage that:

1. persists a reconciliation line in sqlite and verifies point lookup returns it
2. verifies sqlite point lookup returns `None` for an unknown id
3. performs the same round-trip in postgres integration tests when a test URL is available
4. re-runs admin reconciliation tests to confirm the resolve endpoint still behaves correctly

## Out Of Scope

- new indexes or schema changes
- bulk reconciliation lookup APIs
- dashboard summary endpoints
- cache layers for reconciliation reads
