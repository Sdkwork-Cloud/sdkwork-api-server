# Payment Reconciliation Metrics Design

## Scope

This tranche exposes payment reconciliation health as Prometheus-friendly metrics on the admin
metrics endpoint.

It builds directly on the existing admin payment reconciliation summary aggregation.

## Problem

The admin API now exposes JSON summary data for reconciliation anomalies, but monitoring systems
still need a scrape-friendly metric surface.

Without native metrics:

- alerting systems must poll JSON endpoints and reimplement parsing
- reconciliation anomalies remain harder to integrate into existing Prometheus / Grafana stacks
- operators cannot define simple active-anomaly alerts from the standard `/metrics` endpoint

That leaves the payment monitoring model incomplete.

## Approaches Considered

### Approach A: Require external systems to scrape JSON summary endpoints

Pros:

- no new metrics implementation

Cons:

- operationally awkward
- duplicates aggregation logic outside the service
- inconsistent with the existing Prometheus metrics contract

### Approach B: Render reconciliation gauges from the admin metrics endpoint

Pros:

- fits existing `/metrics` scrape workflow
- keeps monitoring logic server-side
- minimal incremental change because summary aggregation already exists

Cons:

- metrics are computed at scrape time
- only available on the stateful admin router variant

Recommended: Approach B.

## Design

### 1. Extend `/metrics` on the stateful admin router

The existing `admin_router_with_state` metrics route will append reconciliation gauges to the
current HTTP metrics payload.

This slice does not change the static `admin_router()` placeholder router because it does not have
store access and is not the production path.

### 2. Emit lifecycle gauges

Expose:

- `sdkwork_payment_reconciliation_total`
- `sdkwork_payment_reconciliation_active_total`
- `sdkwork_payment_reconciliation_resolved_total`
- `sdkwork_payment_reconciliation_latest_updated_at_ms`
- `sdkwork_payment_reconciliation_oldest_active_created_at_ms`

Rules:

- counts always emit, even when zero
- optional timestamps emit only when a value exists

### 3. Emit active reason gauges

Expose:

- `sdkwork_payment_reconciliation_active_reason_total{reason_code="..."}`

Rules:

- only unresolved anomalies contribute
- missing reason codes are emitted as `reason_code="unknown"`

### 4. Reuse summary aggregation

The Prometheus rendering path reuses the existing admin reconciliation summary aggregation so the
JSON summary endpoint and metrics endpoint stay consistent.

## Testing Strategy

Add an admin integration regression that:

1. creates one resolved reconciliation line and one active reconciliation line
2. calls `GET /metrics`
3. asserts the lifecycle gauges are present with the correct values
4. asserts the active reason gauge is present for the unresolved anomaly reason

## Out Of Scope

- background metric caching
- push-based alert notifications
- payment provider routing / failover metrics
- portal-facing metrics endpoints
