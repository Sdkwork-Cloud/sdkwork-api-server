# Native Dynamic Binary Streams Design

**Date:** 2026-03-14

**Status:** Approved by the existing extension runtime and OpenAI-compatible gateway baseline for direct implementation

## Goal

Close the next important native dynamic parity gap by making trusted `native_dynamic` provider extensions capable of handling non-SSE binary stream operations through the existing host-owned stream ABI.

## Why This Batch

The current runtime already supports:

- JSON provider execution through `native_dynamic`
- SSE stream relay for `/v1/chat/completions`
- SSE stream relay for `/v1/responses`

What is still missing is stream parity for binary content paths such as:

- `audio.speech.create`
- `files.content`
- `videos.content`

This is a meaningful architecture gap because the stream ABI is already byte-oriented and content-type aware, but the host mapping still treats those operations as non-streaming invocations.

## Scope

This batch will implement:

1. explicit stream expectation mapping for binary stream operations in the native dynamic host
2. native fixture support for audio, file, and video content streams
3. runtime tests proving native dynamic can emit non-SSE binary output
4. HTTP end-to-end tests proving stateful gateway relay for native dynamic:
   - `/v1/audio/speech`
   - `/v1/files/{file_id}/content`
   - `/v1/videos/{video_id}/content`
5. manifest and documentation updates reflecting binary stream support

This batch will not implement:

- native lifecycle hooks such as `init`, `health`, or `shutdown`
- weighted routing, health-scored failover, or geo affinity
- hot reload or unload safety changes

## Options Considered

### Option A: Keep binary stream operations on the JSON invocation path

Pros:

- fewer mapping changes

Cons:

- architecturally wrong because the host expects `ProviderOutput::Stream`
- prevents native dynamic providers from returning binary bytes cleanly
- leaves the existing generic stream ABI underused

### Option B: Mark stream-returning operations as `expects_stream = true`

Pros:

- aligns invocation semantics with actual provider output
- reuses the existing callback-based stream ABI without widening the contract
- keeps binary and SSE output on one host-owned stream abstraction

Cons:

- requires touching invocation mapping, fixtures, tests, and docs

## Recommendation

Use **Option B**.

The current stream ABI is already generic enough for binary output. The missing piece is not a new transport abstraction, but correct host-side stream dispatch for operations whose natural output is streamed bytes.

## Target Behavior

When a native dynamic provider receives:

- `ProviderRequest::AudioSpeech`
- `ProviderRequest::FilesContent`
- `ProviderRequest::VideosContent`

the host should:

- map the invocation with `expects_stream = true`
- call the stream ABI export instead of the JSON export
- preserve the plugin-selected `content_type`
- relay raw bytes without buffering or JSON wrapping

This keeps runtime behavior consistent with upstream HTTP-based providers and preserves the same `ProviderStreamOutput` abstraction through the gateway and Axum layers.

## Testing Strategy

The batch will be proven through:

1. extension-host runtime tests for audio, file, and video stream operations
2. HTTP route tests for stateful native dynamic relay on the corresponding endpoints
3. manifest parity updates so discovered package manifests match the library-exported capability set

## Follow-On Work

After this batch, the strongest remaining native dynamic gaps are:

1. lifecycle hooks and richer runtime health contracts
2. hot reload or unload safety
3. more advanced routing strategies over multiple provider instances
