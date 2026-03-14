# Stateless Expanded Relay Design

**Date:** 2026-03-14

**Status:** Approved by the user's standing instruction to continue autonomously without pausing for approval checkpoints

## Goal

Extend the stateless gateway runtime so the remaining OpenAI-compatible data-plane families relay to a configured upstream provider instead of defaulting immediately to local emulation.

## Current Problem

The repository already has strong provider abstractions and broad stateful relay coverage, but stateless mode still diverges from the architecture target in one important way:

1. the bootstrap core APIs relay upstream in stateless mode
2. most of the remaining OpenAI-compatible families still return locally emulated responses even when a stateless upstream runtime is explicitly configured

This leaves the stateless runtime internally inconsistent. A library or Tauri host can configure a real upstream endpoint and API key, but many routes still behave like a demo runtime rather than a real gateway edge.

## Options Considered

### Option A: Keep stateless mode narrow and document the remaining emulated families

Pros:

- smallest implementation scope
- no new test surface

Cons:

- preserves a large compatibility gap between stateful and stateless mode
- makes embedded mode less trustworthy for real-world workflows
- keeps the compatibility matrix truthful but unsatisfactory

### Option B: Expand stateless relay route-by-route using the existing provider request abstraction

Pros:

- reuses the current `ProviderRequest` contract and extension host runtime resolution
- significantly reduces `emulated` behavior without adding a second control plane
- keeps local fallback for zero-config usage
- aligns stateless and stateful gateway semantics more closely

Cons:

- touches many HTTP handlers and tests
- requires disciplined consistency across JSON, multipart, and binary stream routes

### Option C: Replace the stateless router with a generic request translation layer

Pros:

- could reduce handler duplication in the long term
- might unify stateful and stateless execution paths more aggressively

Cons:

- too much architectural churn for this batch
- high risk of accidental regressions across many routes
- would delay shipping real compatibility improvements

## Recommendation

Use **Option B**.

The provider layer already knows how to execute these OpenAI-compatible operations. The highest-value move is to wire the stateless router to that capability consistently, not to redesign the HTTP layer again.

## Proposed Capability

Stateless mode should attempt upstream relay first for the following families whenever `StatelessGatewayConfig.upstream` is present:

- `/v1/files`
- `/v1/uploads`
- `/v1/audio/*`
- `/v1/images/*`
- `/v1/moderations`
- `/v1/realtime/sessions`
- `/v1/assistants`
- `/v1/threads`
- `/v1/conversations`
- `/v1/vector_stores`
- `/v1/batches`
- `/v1/fine_tuning/jobs`
- `/v1/webhooks`
- `/v1/evals`
- `/v1/videos`

This includes:

- JSON request and response flows
- multipart request flows
- binary or event-stream response flows for:
  - `/v1/audio/speech`
  - `/v1/files/{file_id}/content`
  - `/v1/videos/{video_id}/content`

## Runtime Semantics

The stateless runtime should keep the existing contract:

1. no database or persisted control plane is required
2. one optional explicit upstream runtime can be injected
3. local compatible fallback remains available when no upstream runtime resolves

The relay decision should follow this order:

1. if no stateless upstream is configured, use local emulation
2. if an upstream is configured but its runtime key does not resolve, fall back locally
3. if an upstream is configured and selected but the upstream call fails, return `502 Bad Gateway`
4. if the upstream succeeds, return the upstream result directly

This matches the current stateless bootstrap behavior and keeps failure semantics explicit.

## Architecture

The design should stay inside the current layering:

- `sdkwork-api-interface-http`
  owns stateless HTTP routing, multipart parsing, and fallback behavior
- `sdkwork-api-provider-core`
  already models the required provider operations
- `sdkwork-api-provider-openai`
  already implements the required JSON, multipart, and stream flows
- `sdkwork-api-provider-openrouter`
  and `sdkwork-api-provider-ollama`
  already delegate through the same OpenAI-compatible adapter
- `sdkwork-api-extension-host`
  already resolves canonical extension IDs and compatibility aliases for stateless runtime keys

No new provider abstraction is needed in this batch.

## Handler Design

The current stateless handlers should move toward one consistent shape:

1. parse the incoming request
2. attempt `relay_stateless_json_request` or `relay_stateless_stream_request`
3. convert an upstream success into the HTTP response immediately
4. map upstream execution failure to a route-specific `502`
5. only execute the current local fallback path when relay returns `Ok(None)`

Where possible, common helper functions should be used so the router does not accumulate one-off error handling per route family.

## Compatibility Matrix Impact

If implemented as designed, the stateless compatibility labels should change from `emulated` to `relay` for the families listed above, while keeping notes that local compatible fallback still exists when no stateless upstream is configured.

This is an execution-truth improvement, not a wording-only documentation change.

## Testing Strategy

Add stateless upstream relay tests for representative behavior in each affected family:

- JSON relay
- multipart relay
- binary stream relay
- nested resource relay such as assistants, threads, conversation items, vector store file batches, and uploads

The tests should verify:

- upstream authorization header propagation
- multipart content type preservation where relevant
- expected upstream JSON or binary payload is returned
- existing local fallback still works when no stateless upstream exists

## Scope

This batch will:

- expand stateless upstream relay to the remaining OpenAI-compatible data-plane families already supported by the provider abstraction
- keep local emulation as the zero-config fallback
- update the compatibility and runtime docs to match the new behavior

This batch will not:

- add stateless quota, billing, or routing policies
- add multi-upstream stateless dispatch
- redesign the provider abstraction
- remove local emulation paths entirely
