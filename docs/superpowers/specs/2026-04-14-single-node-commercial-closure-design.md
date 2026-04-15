# Single-Node Commercial Closure Design

**Date:** 2026-04-14

## Goal

Close the first production-grade slice of `sdkwork-api-router` as a single-node deployment that is commercially usable for one tenant-facing workflow:

`portal register/login -> workspace bootstrap -> API key issuance -> gateway request admission -> usage/billing/audit visibility -> admin traceability -> managed install/start/stop lifecycle`

This slice is the gate for deeper expansion into clustered deployment, multi-region rollout, and desktop embedding.

## Why This Slice First

The repository already contains broad functional coverage across gateway, admin, portal, runtime host, product runtime, billing, marketing, payment, and release tooling. What blocks commercial readiness is no longer raw feature count. The real risk is closure risk:

- the integrated product path must compile and test cleanly
- managed runtime tooling must stay operable on a clean machine
- portal, gateway, billing, and admin must agree on identity and accounting contracts
- audit and billing evidence must be retrievable from the same canonical subject chain

If this single-node path is unstable, every later cluster or region feature multiplies the failure surface.

## Scope

### In Scope

- workspace-wide Rust test baseline restoration
- `router-product-service` single-node runtime contract verification
- portal identity and workspace bootstrap integrity
- portal self-service API key and API key group lifecycle integrity
- gateway admission context integrity, including canonical subject metadata
- usage, billing, and audit evidence continuity across portal/admin boundaries
- managed `build/install/start/stop` runtime tooling behavior needed for single-node operations

### Out of Scope for This Phase

- multi-node clustering
- multi-region topology
- disaster recovery choreography
- desktop embedded runtime hardening
- deep performance capacity work beyond single-node operational correctness

Those remain explicit follow-on phases after the single-node path is trustworthy.

## Architectural Choice

Three implementation routes were considered.

### Option A: Feature-by-feature polish first

Pros:

- visually satisfying progress
- easy to scope small diffs

Cons:

- hides integration defects
- does not prove operability
- leaves release and runtime risk unresolved

### Option B: Release tooling first

Pros:

- fast operational value
- improves install/start confidence

Cons:

- can still ship a runtime whose core portal/gateway/billing contracts are broken

### Option C: Integrated single-node closure first

Pros:

- validates the commercial main path end-to-end
- exposes identity, billing, audit, and runtime mismatches early
- gives a reliable baseline for later cluster and desktop work

Cons:

- broader than a single feature fix
- requires disciplined verification

### Recommendation

Choose Option C. It is the only path that reduces actual commercial delivery risk instead of improving isolated islands.

## System Design

### Runtime Boundary

Single-node production uses `router-product-service` as the deployment entrypoint, with managed scripts responsible for build, install, start, stop, and health verification. The product runtime is the operational truth for this phase, not the standalone development servers.

### Identity Boundary

Portal session identity, workspace scope, gateway request context, and canonical subject metadata must agree. The system already evolved `GatewayRequestContext` to carry canonical identifiers. Any stale test or runtime adapter that still constructs the pre-canonical shape is now contract drift and must be corrected.

### Commercial Evidence Boundary

Portal-facing usage, billing, API key, and workspace views must be derivable from the same subject chain used by gateway admission and admin audit visibility. The phase focuses on preserving that evidence chain rather than adding new business modules.

### Operational Boundary

Single-node operators need a reliable `build -> install -> start -> health -> stop` lifecycle with sensible defaults, deterministic config discovery, writable runtime state, and clear failure surfaces when ports, config, or runtime contracts drift.

## Acceptance Criteria

### Functional

- portal registration and login succeed on a clean runtime
- authenticated portal workspace requests succeed
- portal API key management remains workspace-scoped
- gateway request context compiles and executes with canonical subject support
- portal/admin billing and audit views remain scoped and traceable

### Security

- no request context path drops canonical subject fields by accident
- workspace scoping tests remain enforced
- portal-authenticated flows cannot read foreign tenant/project commercial evidence

### Operational

- workspace Rust test suite reaches a green baseline or remaining failures are isolated and documented as the next blocking tasks
- single-node managed runtime scripts remain the default verification surface

### Delivery

- architecture and execution plan are documented under `docs/superpowers/`
- implementation follows failing-test-first where behavior changes are required

## Immediate Risks Already Observed

- the current workspace test baseline is broken by contract drift in `GatewayRequestContext`
- several integration crates emit unused-code/import warnings, indicating recent merge churn and incomplete post-merge cleanup
- the commercial main path likely contains more hidden drift behind the first compile failure, so recovery must be iterative and evidence-driven

## Implementation Strategy

1. restore the Rust workspace test baseline beginning with the first failing compilation barrier
2. rerun targeted portal/gateway/billing/product-runtime verification after each fix
3. classify follow-on issues as functional correctness, security scope leakage, or operational tooling defects
4. keep changes focused on commercial closure, not broad refactors

## Success Condition

This phase is successful when the repository has a trustworthy single-node commercial baseline: tests pass for the slice, the identity and billing contracts stay coherent, and the product runtime path can be reasoned about as a shippable deployment foundation.
