# Relay Error Envelope Standardization Design

**Date:** 2026-03-14

**Status:** Approved by the user's standing instruction to continue autonomously without pausing for approval checkpoints

## Context

The repository now covers a broad OpenAI-compatible route surface and already has explicit parity and relay design documents for:

- OpenAI surface parity
- stateless expanded relay
- extension-host standardization
- gateway route hardening

The latest hardening batch fixed OpenAI-style error envelopes for the highest-priority core routes:

- `/v1/models`
- `/v1/chat/completions`
- `/v1/responses`
- `/v1/embeddings`

However, a fresh source audit of `sdkwork-api-interface-http` still shows a large number of relay-failure branches returning plain-text `502 Bad Gateway` bodies instead of the OpenAI JSON error envelope.

That means the architecture is still inconsistent:

- some relay-backed routes are OpenAI-compatible on failure
- many other relay-backed routes still break client expectations when upstream execution fails

## Problem

This is now one of the highest-value remaining compatibility gaps because it cuts across almost the entire advertised API surface:

- assistants
- threads and runs
- files and uploads
- images and audio
- fine-tuning
- evals
- videos
- vector stores
- batches
- webhooks
- realtime sessions
- other relay-backed families

The existing design corpus consistently treats SDKWork as an OpenAI-compatible gateway rather than a best-effort proxy. A mixed failure contract across routes weakens that claim and makes client integration behavior inconsistent.

## Options Considered

### Option A: Leave non-core routes as plain-text `502`

Pros:

- no implementation risk
- no broad handler edits

Cons:

- preserves inconsistent client behavior across route families
- undermines the OpenAI-compatibility claim
- keeps tests focused only on a narrow set of endpoints

### Option B: Standardize upstream relay failures across the entire HTTP surface

Pros:

- makes failure behavior consistent for all relay-backed endpoints
- reuses the existing `bad_gateway_openai_response` helper added during core hardening
- aligns with the parity and relay design documents
- improves both stateless and stateful execution modes

Cons:

- requires a broad handler sweep
- needs representative tests so the change is not a blind mechanical edit

### Option C: Introduce route-family-specific error wrappers

Pros:

- allows custom failure semantics per family

Cons:

- adds more branching and duplication
- weakens consistency without a strong reason
- increases long-term maintenance cost

## Recommendation

Use **Option B**.

The project already has the correct shape for a unified OpenAI-compatible error envelope. The missing step is to apply that contract consistently across the rest of the relay-backed route surface.

## Scope

This batch should:

- standardize relay-failure `502` responses across the remaining stateless and stateful HTTP handlers
- keep the existing status code semantics unchanged
- return JSON bodies shaped like `OpenAiErrorResponse`
- cover representative route families with explicit regression tests, including:
  - JSON relay routes
  - multipart relay routes
  - binary or stream relay routes
  - nested resource relay routes

This batch should not:

- redesign local fallback semantics
- remove every `expect(...)` from the HTTP layer
- change successful relay payload passthrough behavior
- change quota or authentication responses

## Architecture

The implementation should stay entirely within the existing layering:

- `sdkwork-api-interface-http`
  standardizes HTTP error responses at the route edge
- `sdkwork-api-contract-openai`
  already provides the `OpenAiErrorResponse` contract
- provider crates remain unchanged
- gateway execution and extension resolution remain unchanged

The key design rule is simple:

1. relay failure keeps returning HTTP `502`
2. route handlers stop returning plain-text bodies
3. all relay failures use the same OpenAI-style JSON envelope helper

## Testing Strategy

Use TDD:

1. add representative failing tests first
2. verify they fail because the current routes still return plain-text `502`
3. standardize the remaining handler branches
4. run focused route tests
5. run package and workspace verification

Representative tests should cover:

- assistants create failure
- audio speech failure before binary response establishment
- file content failure before binary response establishment
- eval route failure as a nested relay-backed family

## Success Criteria

This batch is complete when:

- representative new failure tests pass
- previously fixed core-route failure tests still pass
- `sdkwork-api-interface-http` no longer mixes plain-text and OpenAI-style relay-failure bodies across route families
- workspace verification remains green
