# Usage And Billing Summary Design

**Date:** 2026-03-14

**Status:** Approved by the user's standing instruction to continue autonomously without waiting for checkpoints

## Goal

Close one of the strongest remaining platform gaps by turning usage and billing from list-only admin APIs into first-class summary APIs that the console and operators can consume directly.

## Current Problem

The gateway already records:

- usage records
- ledger entries
- quota policies

The admin control plane currently exposes only list endpoints for those domains. The console therefore computes coarse metrics client-side from raw lists.

That leaves three problems:

1. statistics semantics live in the UI instead of the backend
2. there is no stable operations-facing API for platform summaries
3. quota pressure is visible only indirectly and inconsistently

## Options Considered

### Option A: Keep list APIs only and aggregate in React

Pros:

- smallest code diff
- no backend changes

Cons:

- summary semantics are duplicated in every client
- weak API platform story
- does not improve external or embedded admin integrations

### Option B: Add dedicated admin summary endpoints over existing persisted records

Pros:

- backend owns the aggregation contract
- no schema or migration changes required
- low-risk and high-value
- console and future clients can share one stable summary model

Cons:

- current aggregation remains in-memory over stored lists, not pre-aggregated

### Option C: Add pre-aggregated analytics tables and a separate observability service

Pros:

- strongest long-term scalability
- fits the target service decomposition story

Cons:

- much larger scope
- needs new schema, write paths, and lifecycle concerns
- too heavy for the current gap

## Recommendation

Use **Option B**.

The repository already has the persistence and API boundaries needed to expose summary read models. Adding summary endpoints now materially improves the control plane without creating a second analytics architecture prematurely.

## Proposed API Additions

Add two authenticated admin endpoints:

- `GET /admin/usage/summary`
- `GET /admin/billing/summary`

These endpoints should be JWT-protected in the same way as the existing admin list routes.

## Summary Models

### Usage Summary

Usage summary should include:

- total request count
- distinct project count
- distinct model count
- distinct provider count
- per-project request counts
- per-provider request counts
- per-model request counts

This gives operators an immediate view of gateway traffic shape without transferring raw records into every client.

### Billing Summary

Billing summary should include:

- total ledger entry count
- total booked units
- total booked amount
- active quota policy count
- exhausted project count
- per-project billing posture

Per-project posture should include:

- project ID
- used units
- booked amount
- effective quota policy ID when present
- quota limit units when present
- remaining units when present
- whether the project is currently exhausted

This lets operators identify quota pressure directly from one response.

## Layering

The change should preserve the current DDD-oriented boundaries:

- domain crates define the summary read models
- app crates compute summaries from store-backed records
- interface-admin exposes HTTP handlers
- console types and admin SDK consume the new summary models

No storage trait or database migration changes are needed for the first version because the summaries can be derived from existing persisted records.

## Error Handling

- summary handlers should return `500` on store access failure, consistent with the existing admin handlers
- empty datasets should return successful empty summaries, not errors

## Testing Strategy

Add:

- domain or app-level tests for aggregation behavior
- admin HTTP tests proving the summary endpoints return correct JSON from SQLite-backed state
- console typecheck and build verification after SDK and UI updates

## Scope

This batch will:

- add usage summary models and service functions
- add billing summary models and service functions
- expose both summaries through admin HTTP routes
- expose them in the TypeScript admin SDK and shared console types
- update the usage console page to consume backend summaries

This batch will not:

- add a full metrics exporter
- add Prometheus or OpenTelemetry wiring
- add pre-aggregated analytics tables
- split usage or billing into standalone deployable services yet
