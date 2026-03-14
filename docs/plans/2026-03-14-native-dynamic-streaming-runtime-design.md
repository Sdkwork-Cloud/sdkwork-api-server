# Native Dynamic Streaming Runtime Design

**Date:** 2026-03-14

**Status:** Approved by existing extension architecture baseline for direct implementation

## Goal

Close the most important remaining native dynamic data-plane gap by making `native_dynamic` provider extensions capable of producing real chat/SSE stream output instead of only JSON responses.

## Why This Batch

The gateway already supports:

- built-in provider stream relay through OpenAI-compatible HTTP upstreams
- connector runtime supervision and relay through external HTTP runtimes
- native dynamic JSON execution for trusted provider extensions

What is still missing is stream parity for the in-process plugin runtime. Until a `native_dynamic` provider can satisfy `chat.completions.stream`, the extension runtime model is still incomplete for the most important real-time OpenAI-compatible path.

## Scope

This batch will implement:

1. a provider-agnostic stream output model in `sdkwork-api-provider-core`
2. host-side native dynamic stream execution through a callback-based C ABI
3. gateway and HTTP integration for native dynamic chat SSE relay
4. fixture coverage proving a native dynamic plugin can emit real SSE frames

This batch will not implement:

- generic binary stream ABI for file, audio, or video content
- hot reload or unload safety changes
- lifecycle hooks such as `init` or `shutdown`
- weighted and health-scored routing

## Options Considered

### Option A: Keep `reqwest::Response` as the only stream type

Teach the native plugin runtime to somehow synthesize a `reqwest::Response`.

Pros:

- smaller surface change

Cons:

- leaks HTTP client implementation details into the plugin boundary
- blocks non-HTTP-native runtimes from participating cleanly
- produces the wrong abstraction for an extension host

### Option B: Introduce a host-owned stream output abstraction

Replace the `reqwest::Response`-only stream path with a provider-agnostic stream object carrying:

- content type
- a byte stream owned by the provider runtime

Then adapt HTTP-based providers into that shape and let native dynamic plugins write chunks through a host callback ABI.

Pros:

- clean runtime abstraction
- supports both HTTP upstreams and in-process plugins
- preserves real streaming semantics
- is a natural base for future connector and native lifecycle evolution

Cons:

- requires touching provider-core, provider adapters, gateway, and HTTP interface

### Option C: Return a fully buffered SSE body from the plugin

Let the plugin return one big string containing all SSE frames.

Pros:

- easiest to wire

Cons:

- not real streaming
- undermines latency and backpressure behavior
- fails the architectural goal

## Recommendation

Use **Option B**.

The host must own the stream abstraction so the extension runtime stays pluggable and independent from a particular transport client.

## Target Stream Model

`ProviderOutput` should stop coupling streams to `reqwest::Response`.

Instead, stream-capable provider execution should return a host-owned output shape with:

- `content_type`
- a byte stream suitable for Axum `Body::from_stream`

HTTP-based providers will adapt upstream `reqwest::Response::bytes_stream()` into this shape.

Native dynamic providers will produce the same shape by writing chunks into a host-managed channel.

## Native Dynamic Stream ABI

The stream ABI should remain minimal and explicit.

### New exported symbol

- `sdkwork_extension_provider_execute_stream_json`

### Host-to-plugin contract

The host passes:

- the same JSON invocation envelope already used for JSON execution
- a `ProviderStreamWriter` callback table owned by the host

### Callback responsibilities

The plugin may:

- set content type before the first chunk
- write arbitrary byte chunks
- stop writing if the host callback reports the receiver has been closed

### Completion result

The stream function returns a JSON result describing:

- `streamed`
- `unsupported`
- `error`

This keeps error reporting explicit without coupling the ABI to Rust async traits.

## Host Execution Model

The extension host will:

1. detect whether the native dynamic library exports the stream symbol
2. build a host-managed channel for bytes
3. spawn a blocking task or thread that invokes the plugin stream export
4. expose the receiver side as the unified provider stream output
5. fail fast if the plugin reports `unsupported` or `error` before the stream starts

This keeps the plugin ABI synchronous while still letting the HTTP layer emit data incrementally.

## Compatibility Boundary

This batch intentionally limits the native dynamic stream ABI to SSE-compatible provider operations.

Concretely, the first supported path is:

- `chat.completions.stream`

The design is still generic enough to evolve later, but audio/file/video content will stay on the existing transport-specific stream path for now.

## Testing Strategy

The implementation will be proven through:

1. ABI contract tests for the new stream symbol and result envelope
2. host tests proving a native dynamic fixture can emit chunked SSE bytes
3. gateway or HTTP route tests proving a signed native dynamic extension can relay a real stream through `/v1/chat/completions`

## Follow-On Work

After this batch, the most logical next steps are:

1. native dynamic lifecycle hooks
2. generic binary stream support for file/audio/video content
3. weighted and health-scored routing over multiple provider instances
4. richer extension capability and health observability
