# Core Route Hardening Design

**Date:** 2026-03-14

**Status:** Approved by the user's standing instruction to proceed autonomously

## Context

The repository already passes its current test suite and exposes a large OpenAI-compatible API surface, but a fresh review of the highest-traffic runtime paths surfaced two concrete issues:

- several OpenAI-compatible HTTP handlers return plain-text `502 Bad Gateway` bodies on upstream relay failures instead of the structured OpenAI error envelope
- the gateway rebuilds the configured extension host on the request path for every provider execution, including repeated extension discovery and trust checks

These issues do not break the current green test suite, but they weaken real-world compatibility and add avoidable per-request overhead.

## Goal

Harden the first-priority core API paths so they return OpenAI-compatible error payloads and avoid redundant extension-host reconstruction on hot request paths.

## Scope

This batch is intentionally limited to the core API paths the user previously prioritized:

- `/v1/models`
- `/v1/chat/completions`
- `/v1/responses`
- `/v1/embeddings`
- chat and responses streaming behavior related to those routes

## Design

### 1. Error Envelope Compatibility

For the scoped core routes, upstream relay failures should return:

- the same HTTP status they use today for upstream failure handling
- a JSON body shaped like `OpenAiErrorResponse`

This keeps compatibility with OpenAI-style client expectations while preserving the existing failure semantics.

Implementation approach:

- add a small response helper in `sdkwork-api-interface-http`
- use it in the scoped stateless and stateful handlers
- keep the current fallback-to-local behavior unchanged

### 2. Extension Host Reuse

The gateway should not rediscover extensions and rebuild the host on every provider request when the effective extension discovery configuration has not changed.

Implementation approach:

- introduce a lightweight cache inside `sdkwork-api-app-gateway`
- key the cache from the effective discovery-policy inputs currently read from the process environment
- rebuild the host only when that cache key changes

This keeps behavior deterministic for tests and local configuration changes while removing redundant work for repeated requests.

### 3. Testing Strategy

Use TDD for both changes:

- add failing HTTP route tests proving upstream relay failures return OpenAI-style JSON error envelopes for the scoped endpoints
- add failing gateway tests proving repeated runtime execution reuses the same cached extension host for a stable configuration

## Non-Goals

This batch will not:

- rewrite every `BAD_GATEWAY` path in the entire interface layer
- remove every `expect(...)` in the HTTP crate
- redesign the extension runtime loading model
- add background invalidation or filesystem watching for extension cache refresh
