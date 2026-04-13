# Alerts And SLOs

This document defines the minimum telemetry interpretation and alert posture for running
`sdkwork-api-router` as a commercial API routing service.

## Core SLOs

- Gateway availability:
  Successful responses should remain above `99.9%` over 30 days for paid production traffic.
- Gateway latency:
  `p95` gateway request latency should remain below `1s` for synchronous text routes and below
  `3s` for media submit routes over a 1 hour rolling window.
- Payment callback timeliness:
  `p95` payment callback processing latency should remain below `60s` from provider delivery to
  internal settlement completion.
- Throttling posture:
  Sustained throttling for the same project, API key group, or provider should remain exceptional
  rather than steady-state background behavior.
- Settlement correctness:
  Hold failures, callback replays, and settlement replays should be auditable and should not create
  duplicate ledger movement.

## Metric Families

- `sdkwork_http_requests_total`
  Includes `service`, `method`, `route`, `status`, `tenant`, `model`, `provider`,
  `billing_mode`, `retry_outcome`, `failover_outcome`, and `payment_outcome`.
- `sdkwork_http_request_duration_ms_bucket|sum|count`
  Histogram-style request latency for `gateway`, `admin`, and `portal`.
- `sdkwork_provider_execution_total`
  Provider attempt outcomes with `route`, `tenant`, `model`, `provider`, `billing_mode`,
  `retry_outcome`, `failover_outcome`, and `result`.
- `sdkwork_provider_execution_duration_ms_bucket|sum|count`
  Histogram-style upstream execution latency.
- `sdkwork_payment_callbacks_total`
  Payment callback outcomes by provider, workspace/project tenant, and settlement result.
- `sdkwork_commercial_events_total`
  Structured event counter for `hold_failure`, `settlement_replay`, `failover_activation`,
  `callback_replay`, and `throttling`.

The dynamic label sets are cardinality-limited in-process. Once a label family exceeds its
configured budget, new values collapse to `other` instead of exploding the series count.

## Recommended Alerts

### Gateway Latency

Trigger when the 5 minute `p95` latency for the gateway exceeds SLO:

```promql
histogram_quantile(
  0.95,
  sum by (le, route) (
    rate(sdkwork_http_request_duration_ms_bucket{service="gateway"}[5m])
  )
) > 1000
```

### Provider Error Rate

Trigger when retryable or terminal provider failures exceed `5%` for 10 minutes:

```promql
sum by (provider, route) (
  rate(sdkwork_provider_execution_total{service="gateway",result=~"retryable_failure|terminal_failure"}[10m])
)
/
sum by (provider, route) (
  rate(sdkwork_provider_execution_total{service="gateway"}[10m])
) > 0.05
```

### Failover Activation Surge

Trigger when a provider begins forcing abnormal failover volume:

```promql
sum by (provider, route) (
  increase(sdkwork_commercial_events_total{service="gateway",event_kind="failover_activation"}[15m])
) > 20
```

### Payment Callback Replay

Trigger when duplicate callback volume is above a normal background threshold:

```promql
sum by (provider, tenant) (
  increase(sdkwork_payment_callbacks_total{service="portal",payment_outcome="duplicate"}[15m])
) > 3
```

### Payment Callback Failure Or Lag

Trigger when failed callbacks appear or total callback traffic drops while checkout traffic exists:

```promql
sum by (provider, tenant) (
  increase(sdkwork_payment_callbacks_total{service="portal",payment_outcome="failed"}[15m])
) > 0
```

Operationally, pair this with an application-side queue age or provider delivery timestamp check.
If the provider can emit webhook delivery timestamps, alert when end-to-end callback lag exceeds
`60s`.

### Rate-Limit Saturation

Trigger when throttling becomes sustained rather than bursty:

```promql
sum by (tenant, route) (
  increase(sdkwork_commercial_events_total{service="gateway",event_kind="throttling"}[10m])
) > 25
```

### Hold Failure

Trigger immediately because hold capture or release failures risk billing correctness:

```promql
sum by (route, tenant) (
  increase(sdkwork_commercial_events_total{service="gateway",event_kind="hold_failure"}[5m])
) > 0
```

### Settlement Replay

Trigger when fulfilled orders are repeatedly replayed in a way that needs finance/operator review:

```promql
sum by (tenant) (
  increase(sdkwork_commercial_events_total{event_kind="settlement_replay"}[30m])
) > 5
```

## Dashboards

- Gateway dashboard:
  Request volume, latency histogram, provider execution latency, provider failure ratio,
  failover activation counts, throttling counts.
- Billing and payments dashboard:
  Payment callback outcome counts, callback replay counts, settlement replay counts,
  hold failure counts, canonical balance movement, recharge success and failure rates.
- Workspace operations dashboard:
  Top throttled tenants, top failing providers, rate-limit saturation by route,
  callback duplicate concentration by provider.

## Operator Guidance

- Treat `hold_failure` as finance-grade incidents because balance correctness is at risk.
- Treat `callback_replay` as integration incidents when they spike, even if idempotency is working.
- Sustained `failover_activation` means the router is protecting customer traffic, but the provider
  fleet is degraded and should be investigated immediately.
- Repeated `settlement_replay` on the same tenant/project usually indicates an upstream payment or
  order-state coordination issue rather than healthy retry behavior.
