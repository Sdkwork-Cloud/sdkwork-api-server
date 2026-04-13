# Payment Reconciliation Resolution Design

## Scope

This tranche closes the operator loop for payment anomalies that are already detected through
reconciliation records.

The goal is:

- let admin operators mark payment reconciliation lines as resolved
- persist a resolution timestamp for auditability
- keep anomaly visibility in the admin payment surfaces without inventing a new case-management
  subsystem

This builds on the existing provider-conflict and over-capture audit slices.

## Problem

The current system can create reconciliation evidence for:

- provider transaction reference conflicts
- refund provider conflicts
- over-capture amount mismatches

But those lines are operationally incomplete:

- they can be listed, but not closed
- there is no `updated_at_ms` on reconciliation lines
- operators cannot distinguish between an active anomaly and one that was already investigated and
  remediated

That leaves the payment operating model stuck at "detect only", which is below commercial
standards.

## Approaches Considered

### Approach A: Build a separate incident/case table

Pros:

- rich workflow potential

Cons:

- introduces a second anomaly model immediately
- too much schema and UI surface for this slice

### Approach B: Reuse reconciliation lines as the anomaly lifecycle record

Pros:

- minimal change set
- aligns with existing admin list views
- enough to get from detection to closure

Cons:

- less expressive than a full incident system

Recommended: Approach B.

## Design

### 1. Add `updated_at_ms` to reconciliation lines

`ReconciliationMatchSummaryRecord` gains `updated_at_ms`.

Rules:

- when a line is first created, `updated_at_ms` is set to the same value as `created_at_ms`
- when a line is resolved, `updated_at_ms` is set to the resolution timestamp

This gives operators both the original anomaly time and the latest state-change time.

### 2. Resolve endpoint in admin

Add an authenticated admin endpoint:

- `POST /admin/payments/reconciliation-lines/{reconciliation_line_id}/resolve`

Behavior:

- load the reconciliation line by id from the existing store
- if not found, return `404`
- if already `resolved`, return the existing line unchanged
- otherwise update:
  - `match_status = resolved`
  - `updated_at_ms = resolved_at_ms` from request, or server time if omitted

The original `reason_code`, amounts, and foreign references remain unchanged.

### 3. Preserve one anomaly model

This slice does not add:

- custom resolution notes
- separate incident assignment
- queue ownership or SLA tracking

It uses the reconciliation line itself as the operator-visible lifecycle object.

### 4. Sorting and visibility

Admin payment reconciliation listing continues to return all lines, including resolved ones.

Resolved lines remain visible so audit history is preserved. Consumers can distinguish them using:

- `match_status`
- `updated_at_ms`

## Testing Strategy

Add regression coverage for:

1. schema/store:
   - reconciliation lines include `updated_at_ms`
   - sqlite/postgres round-trips preserve `updated_at_ms`
2. admin resolution:
   - create an over-capture reconciliation line
   - call resolve endpoint
   - returned line status becomes `resolved`
   - `updated_at_ms` equals the requested resolution timestamp
   - reason code and amounts remain unchanged
3. admin idempotency:
   - repeated resolve call returns the already resolved line without reopening or corrupting it

## Out Of Scope

- rich operator case workflow
- resolution comments / attachments
- assignment queues or escalation rules
- portal exposure of reconciliation anomalies
