# Payment Reconciliation Queue Design

## Scope

This tranche improves the admin reconciliation list so operators can work payment anomalies as
an actual queue instead of a flat audit dump.

It builds directly on the existing visibility and resolution slices:

- reconciliation lines are already persisted
- admin can already list them
- admin can already mark a line as resolved

## Problem

The current `GET /admin/payments/reconciliation-lines` endpoint is still below commercial
operations standards:

- it returns active anomalies and resolved history in one undifferentiated list
- it sorts only by `created_at_ms`, which lets already-resolved incidents crowd out the current
  work queue
- operators cannot request a focused active-only or resolved-only view
- callers can only consume the full history and filter client-side

That is workable for low volume debugging, but weak for real payment operations where anomaly
queues need a clear "what still needs action" surface.

## Approaches Considered

### Approach A: Introduce a separate reconciliation queue table or incident view

Pros:

- could model workflow state explicitly
- leaves room for richer case management later

Cons:

- duplicates data that already exists on reconciliation lines
- adds schema and synchronization complexity immediately
- too heavy for the next incremental payment hardening slice

### Approach B: Add lifecycle filtering and queue ordering on top of reconciliation lines

Pros:

- no schema expansion
- leverages the existing `match_status` plus `updated_at_ms`
- gives operators an actionable queue view now

Cons:

- still not a full incident management system
- filtering remains in the admin interface layer for this tranche

Recommended: Approach B.

## Design

### 1. Add a lifecycle query to the admin list endpoint

Extend:

- `GET /admin/payments/reconciliation-lines`

With an optional query parameter:

- `lifecycle=all|active|resolved`

Rules:

- omitted lifecycle defaults to `all`
- `active` returns every line whose `match_status != resolved`
- `resolved` returns every line whose `match_status == resolved`
- unknown lifecycle values return `400 Bad Request`

This keeps the API explicit and avoids silently returning the wrong queue when a caller misspells
the filter.

### 2. Make the default list behave like an operator queue

When listing reconciliation lines, order them by:

1. unresolved lines before resolved lines
2. `updated_at_ms DESC`
3. `created_at_ms DESC`
4. `reconciliation_line_id DESC`

This keeps active anomalies at the top even when resolved items were updated more recently due to
operator cleanup.

### 3. Keep the storage layer unchanged in this slice

This tranche intentionally reuses the existing store method:

- `list_all_reconciliation_match_summary_records`

Filtering and ordering are applied in the admin interface layer. That keeps the change narrow and
avoids premature persistence API expansion before pagination or dashboard summaries are designed.

### 4. Preserve audit history and queue ergonomics together

The endpoint still supports a full historical view through `lifecycle=all`, but active items are
surfaced first so the same endpoint can drive both:

- operator triage screens
- audit/review pages

## Testing Strategy

Add admin API regressions that:

1. create one reconciliation line and resolve it
2. create another reconciliation line that remains active
3. assert the default list returns the active line before the resolved line
4. assert `lifecycle=active` returns only the active line
5. assert `lifecycle=resolved` returns only the resolved line
6. assert an invalid lifecycle value returns `400 Bad Request`

## Out Of Scope

- pagination or cursoring
- dashboard counts and summary endpoints
- assignment / ownership workflow
- portal exposure of reconciliation anomalies
- database-level filtered reconciliation queries
