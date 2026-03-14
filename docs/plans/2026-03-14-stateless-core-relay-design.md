# Stateless Core Relay Design

**Date:** 2026-03-14

**Status:** Approved by the user's standing instruction to continue autonomously without pausing for approval checkpoints

## Goal

Replace the stateless gateway's hardcoded pseudo-tenant behavior with an explicit stateless runtime contract, and let the core bootstrap APIs relay to a configured upstream provider when stateless runtime settings are present.

## Current Problem

The repository now has strong stateful relay behavior, but stateless mode still has two correctness gaps:

1. `gateway_router()` hardcodes `"tenant-1"` and `"project-1"` across a large number of local fallback handlers.
2. The most important bootstrap APIs in stateless mode remain purely `emulated` even when a library consumer or embedded runtime could provide a single upstream endpoint and API key directly.

This leaves the stateless runtime below the original architecture target. It is compatible enough for demos, but not explicit enough for serious embedded or library-hosted usage.

## Options Considered

### Option A: Replace hardcoded literals with named constants only

Pros:

- smallest diff
- low implementation risk

Cons:

- still not a real runtime contract
- keeps stateless mode opaque and unconfigurable
- does nothing to reduce `emulated` behavior

### Option B: Introduce a typed stateless runtime contract plus optional upstream relay for core APIs

Pros:

- removes hidden hardcoding
- keeps stateless mode compatible without forcing a database
- reduces `emulated` behavior where it matters most
- gives embedded and library consumers a stable integration surface

Cons:

- touches router composition and several handlers
- requires focused provider-adapter additions for model listing and retrieval

### Option C: Build full stateless multi-provider routing with policy support

Pros:

- closest to end-state gateway architecture
- would unify stateful and stateless routing behavior further

Cons:

- too much scope for one iteration
- duplicates control-plane concepts without persistent backing
- risks overbuilding before the simpler stateless contract exists

## Recommendation

Use **Option B**.

The most valuable next step is to make stateless mode explicit and useful, not to turn it into a second full control plane. A typed stateless runtime contract plus optional single-upstream relay for the bootstrap APIs creates immediate practical value while staying aligned with the current architecture.

## Proposed Capability

Add a `StatelessGatewayConfig` contract to the HTTP interface layer with:

- explicit stateless tenant ID
- explicit stateless project ID
- optional upstream runtime settings:
  - adapter kind
  - base URL
  - API key

Behavior:

1. `gateway_router()` continues to work with a default stateless config.
2. library consumers can call a new constructor to supply a custom stateless config.
3. stateless handlers stop using hardcoded `"tenant-1"` and `"project-1"`.
4. core APIs attempt upstream relay when stateless upstream settings exist:
   - `/v1/models`
   - `/v1/models/{model_id}`
   - `/v1/chat/completions`
   - `/v1/completions`
   - `/v1/responses`
   - `/v1/embeddings`
   - chat and responses streaming
5. when no stateless upstream is configured, the existing compatible local emulation remains the fallback

## Architecture

The implementation should stay layered:

- `sdkwork-api-interface-http`
  owns the stateless router contract and stateless execution state
- `sdkwork-api-provider-core`
  continues to define generic provider execution requests
- `sdkwork-api-provider-openai`
  gains model list and retrieve support needed by stateless bootstrap relay
- `sdkwork-api-provider-openrouter`
  and `sdkwork-api-provider-ollama`
  delegate the same model support through their existing OpenAI-compatible wrapper

This keeps stateless runtime behavior inside the interface layer while still reusing the provider abstraction that already powers the broader gateway execution model.

## Stateless Identity Contract

The default stateless identity should become explicit rather than implicit:

- default tenant ID: `sdkwork-stateless`
- default project ID: `sdkwork-stateless-default`

These values are synthetic runtime scope markers, not persisted control-plane records.

## Upstream Contract

The first stateless upstream contract should remain intentionally narrow:

- one upstream at a time
- one adapter kind
- one base URL
- one API key

This is enough to support:

- embedded local bootstrap against a vendor endpoint
- simple library-mode usage
- test harnesses that need realistic upstream behavior without a database

It intentionally does not add:

- multi-provider routing
- quota enforcement
- credential persistence
- extension installation state

Those remain stateful-mode responsibilities.

## Error Handling

- invalid or missing stateless upstream settings should silently fall back to local emulation
- upstream relay failures should return `502 Bad Gateway` only when stateless relay was explicitly configured and selected
- model list and retrieval should fall back locally only when no stateless upstream exists, not after an upstream execution failure
- tracing and request ID behavior should remain unchanged for stateless requests

## Testing Strategy

Add tests for:

- typed stateless config defaults
- stateless config loading behavior when custom values are supplied programmatically
- stateless `/v1/models` relay through a mock OpenAI-compatible upstream
- stateless `/v1/chat/completions` JSON relay
- stateless `/v1/chat/completions` SSE relay
- stateless `/v1/responses` JSON and SSE relay
- stateless `/v1/embeddings` relay
- fallback behavior when no stateless upstream is configured

## Scope

This batch will:

- add a typed stateless runtime contract
- remove hidden hardcoded tenant and project literals from stateless handlers
- add optional stateless upstream relay for the bootstrap core APIs
- document the new stateless relay capability

This batch will not:

- add stateless multi-provider routing
- persist stateless credentials
- add stateless quota or billing
- remove the existing local emulation fallback
