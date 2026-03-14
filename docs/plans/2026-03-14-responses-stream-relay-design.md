# Responses Stream Relay Design

**Date:** 2026-03-14

**Status:** Approved by the existing OpenAI-compatible gateway baseline for direct implementation

## Goal

Close the next major OpenAI-compatibility gap by making `/v1/responses` support real streaming relay through the stateful gateway, provider adapters, and extension dispatch contract.

## Why This Batch

The project already supports:

- JSON relay for `/v1/responses`
- streaming relay for `/v1/chat/completions`
- a provider-owned stream abstraction that works for both HTTP upstreams and native dynamic plugins

What is still missing is the same streaming behavior for `/v1/responses`, even though the request contract already carries `stream: Option<bool>`.

This leaves one of the primary OpenAI-compatible inference APIs incomplete relative to the original project scope.

## Scope

This batch will implement:

1. `ProviderRequest::ResponsesStream`
2. upstream stream execution for `/v1/responses` in the OpenAI-compatible provider adapters
3. stateful and stateless HTTP handling for `/v1/responses` SSE output
4. extension-host request mapping so native dynamic or future connector runtimes can handle response streams through the same contract
5. deterministic tests for provider, gateway, and HTTP route behavior

This batch will not implement:

- specialized OpenAI Responses event modeling beyond byte-for-byte SSE passthrough
- binary response streaming for non-SSE payloads
- weighted routing or stream-aware failover policies

## Options Considered

### Option A: Overload the existing `Responses` request variant

Let providers inspect `request.stream` and decide whether to return JSON or stream output.

Pros:

- smaller enum surface

Cons:

- mixes transport mode with operation identity
- weakens type guarantees in the provider boundary
- makes gateway call sites ambiguous

### Option B: Introduce a dedicated `ResponsesStream` request variant

Mirror the existing `ChatCompletions` and `ChatCompletionsStream` split.

Pros:

- keeps stream-vs-JSON dispatch explicit
- aligns with the existing provider-core architecture
- makes extension ABI mapping straightforward

Cons:

- requires touching provider-core, providers, gateway, and HTTP interface

## Recommendation

Use **Option B**.

The provider boundary already treats streaming as a first-class execution mode. `responses` should follow the same pattern as `chat.completions`.

## Target Behavior

When `CreateResponseRequest.stream == Some(true)`:

- HTTP upstream providers should relay `text/event-stream` without buffering
- the stateful gateway should persist usage and return the upstream stream body as-is
- the stateless demo router should emit a deterministic local SSE fallback
- extension dispatch should mark the provider invocation as `expects_stream = true`

When `stream` is absent or false, the current JSON behavior remains unchanged.

## Testing Strategy

The batch will be proven through:

1. provider adapter tests showing `/v1/responses` can return a `ProviderStreamOutput`
2. gateway/HTTP route tests showing a stateful upstream provider can relay SSE through `/v1/responses`
3. extension-host mapping tests showing `ResponsesStream` becomes a stream-capable provider invocation

## Follow-On Work

After this batch, the next strongest remaining compatibility gaps are:

1. native dynamic lifecycle hooks
2. generic binary stream parity for non-SSE content
3. more advanced routing strategies such as weighted and health-scored failover
