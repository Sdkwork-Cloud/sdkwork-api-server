# Payment Reconciliation Summary Design

## Scope

This tranche adds an operator-facing summary surface for payment reconciliation anomalies.

It builds on the existing slices that already provide:

- persisted reconciliation evidence
- admin list visibility
- lifecycle queue filtering
- operator resolution

## Problem

The payment subsystem can now detect, list, and resolve anomalies, but it still lacks a compact
monitoring view.

That leaves several commercial operations gaps:

- dashboards cannot query active anomaly counts without downloading the full list
- alerting integrations cannot quickly determine whether unresolved payment anomalies exist
- operators cannot see which reconciliation reasons are currently dominant
- the admin API still exposes raw rows better than monitoring-friendly aggregates

This is a gap between "queue management" and "operational observability".

## Approaches Considered

### Approach A: Require monitoring systems to fetch the full reconciliation list and aggregate client-side

Pros:

- no new endpoint

Cons:

- pushes needless work to every consumer
- inflates payload size as history grows
- creates inconsistent summary logic across dashboards and tooling

### Approach B: Add a server-side reconciliation summary endpoint in the admin API

Pros:

- compact and stable monitoring surface
- keeps anomaly aggregation logic in one place
- leverages existing reconciliation lifecycle state

Cons:

- introduces a new response DTO
- still aggregates in the admin layer for this tranche

Recommended: Approach B.

## Design

### 1. Add an admin reconciliation summary endpoint

Expose:

- `GET /admin/payments/reconciliation-summary`

The endpoint is authenticated the same way as the existing admin payment routes.

### 2. Return monitoring-oriented lifecycle totals

The response returns:

- `total_count`
- `active_count`
- `resolved_count`
- `latest_updated_at_ms`
- `oldest_active_created_at_ms`

Rules:

- `active_count` means every line whose `match_status != resolved`
- `resolved_count` means every line whose `match_status == resolved`
- `latest_updated_at_ms` is `null` when no reconciliation lines exist
- `oldest_active_created_at_ms` is `null` when no active anomalies exist

### 3. Return per-reason active anomaly breakdown

The response also includes:

- `active_reason_breakdown: Vec<...>`

Each item contains:

- `reason_code`
- `count`
- `latest_updated_at_ms`

Rules:

- lines without `reason_code` are grouped under `"unknown"`
- only active lines contribute to the breakdown
- items are sorted by:
  1. `count DESC`
  2. `latest_updated_at_ms DESC`
  3. `reason_code ASC`

This keeps the response focused on "what still needs action now".

### 4. Keep aggregation in the admin layer for this slice

This tranche intentionally reuses:

- `list_all_reconciliation_match_summary_records`

The admin interface aggregates from persisted rows in memory. That keeps the change narrow while
still creating a stable summary contract for future dashboard or metrics plumbing.

## Testing Strategy

Add admin regressions that:

1. create one resolved reconciliation line and one active reconciliation line
2. call `GET /admin/payments/reconciliation-summary`
3. assert lifecycle totals are correct
4. assert `latest_updated_at_ms` tracks the newest row update
5. assert `oldest_active_created_at_ms` tracks the oldest unresolved anomaly
6. assert the active reason breakdown includes only unresolved anomalies
7. assert empty state returns zero counts and `null` optional timestamps

## Out Of Scope

- Prometheus metric emission from payment reconciliation state
- persistence-layer summary queries
- portal reconciliation summaries
- SLA timers, assignment queues, or notification fan-out
