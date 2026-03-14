# Native Dynamic Streaming Runtime Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make trusted `native_dynamic` provider extensions capable of relaying real chat/SSE stream output through the gateway.

**Architecture:** Introduce a host-owned provider stream abstraction, adapt HTTP-based providers into that abstraction, and add a callback-based native dynamic stream ABI so in-process plugins can write byte chunks into a host-managed stream channel.

**Tech Stack:** Rust, Axum, reqwest, tokio, tokio-stream, bytes, libloading, serde

---

### Task 1: Add failing tests for native dynamic streaming

**Files:**
- Modify: `crates/sdkwork-api-extension-abi/tests/abi_contract.rs`
- Modify: `crates/sdkwork-api-extension-host/tests/native_dynamic_runtime.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/chat_stream_route.rs`
- Modify: `crates/sdkwork-api-ext-provider-native-mock/src/lib.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- the ABI crate serializes the stream completion envelope
- the host can execute a native dynamic streaming invocation and collect emitted SSE bytes
- the HTTP gateway can relay a stateful `/v1/chat/completions` SSE response through a signed native dynamic extension

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-extension-abi --test abi_contract -q`
- `cargo test -p sdkwork-api-extension-host --test native_dynamic_runtime -q`
- `cargo test -p sdkwork-api-interface-http --test chat_stream_route stateful_chat_stream_route_relays_to_native_dynamic_provider -- --exact`

Expected: FAIL because the current ABI has no stream symbol, the host has no stream execution path, and the HTTP layer is still coupled to `reqwest::Response`.

### Task 2: Introduce the provider stream abstraction

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/sdkwork-api-provider-core/Cargo.toml`
- Modify: `crates/sdkwork-api-provider-core/src/lib.rs`
- Modify: `crates/sdkwork-api-provider-openai/src/lib.rs`
- Modify: `crates/sdkwork-api-provider-openrouter/src/lib.rs`
- Modify: `crates/sdkwork-api-provider-ollama/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`

**Step 1: Add a unified provider stream output type**

Define a host-owned stream object carrying:

- content type
- boxed byte stream

Replace `ProviderOutput::Stream(reqwest::Response)` with this unified type.

**Step 2: Adapt HTTP-based providers**

Wrap `reqwest::Response::bytes_stream()` into the new provider stream abstraction for:

- chat completion SSE
- speech/file/video passthrough content

**Step 3: Update gateway and HTTP interface**

Make the app gateway return the new stream output type and teach the HTTP layer to build `axum::body::Body` from the provider stream abstraction.

**Step 4: Run focused tests**

Run:

- `cargo test -p sdkwork-api-provider-openai -q`
- `cargo test -p sdkwork-api-app-gateway -q`
- `cargo test -p sdkwork-api-interface-http --test chat_stream_route -q`

Expected: PASS

### Task 3: Add native dynamic stream ABI and host execution

**Files:**
- Modify: `crates/sdkwork-api-extension-abi/src/lib.rs`
- Modify: `crates/sdkwork-api-extension-abi/tests/abi_contract.rs`
- Modify: `crates/sdkwork-api-extension-host/Cargo.toml`
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`
- Modify: `crates/sdkwork-api-ext-provider-native-mock/src/lib.rs`
- Modify: `crates/sdkwork-api-extension-host/tests/native_dynamic_runtime.rs`

**Step 1: Define the stream ABI**

Add:

- stream symbol constant
- `ProviderStreamWriter`
- stream completion result envelope

**Step 2: Implement host-side stream execution**

Teach the host to:

- resolve the optional stream export from the library
- invoke it on a blocking thread
- bridge plugin chunks into the new provider stream abstraction
- report early `unsupported` or `error` states cleanly

**Step 3: Implement fixture stream behavior**

Update the native mock plugin so `chat.completions.create` with `expects_stream = true` emits deterministic SSE frames.

**Step 4: Run focused tests**

Run:

- `cargo test -p sdkwork-api-extension-abi --test abi_contract -q`
- `cargo test -p sdkwork-api-extension-host --test native_dynamic_runtime -q`

Expected: PASS

### Task 4: Wire stateful HTTP relay for native dynamic chat streams

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/chat_stream_route.rs`
- Modify: `README.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/api/compatibility-matrix.md`

**Step 1: Add an end-to-end native dynamic stream route test**

Prove that:

- a signed native dynamic extension can be discovered and loaded
- the gateway can route a streaming chat completion to it
- the HTTP route returns `text/event-stream`
- the body contains expected SSE frames and `[DONE]`

**Step 2: Update docs**

Reflect that native dynamic runtime now supports:

- JSON execution
- chat/SSE streaming

while binary stream parity and lifecycle hooks remain future work.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-interface-http --test chat_stream_route stateful_chat_stream_route_relays_to_native_dynamic_provider -- --exact`
- `cargo test -p sdkwork-api-interface-http -q`

Expected: PASS

### Task 5: Run verification and commit

**Files:**
- Modify all implementation and documentation files above

**Step 1: Run verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `$env:CARGO_BUILD_JOBS='1'; cargo clippy --no-deps -p sdkwork-api-provider-core -p sdkwork-api-extension-abi -p sdkwork-api-extension-host -p sdkwork-api-app-gateway -p sdkwork-api-interface-http --all-targets -- -D warnings`
- `$env:CARGO_BUILD_JOBS='1'; cargo test --workspace -q -j 1`

Expected: PASS

**Step 2: Commit**

```bash
git add Cargo.toml Cargo.lock README.md docs/architecture/runtime-modes.md docs/api/compatibility-matrix.md docs/plans/2026-03-14-native-dynamic-streaming-runtime-design.md docs/plans/2026-03-14-native-dynamic-streaming-runtime-implementation.md crates/sdkwork-api-provider-core crates/sdkwork-api-provider-openai crates/sdkwork-api-provider-openrouter crates/sdkwork-api-provider-ollama crates/sdkwork-api-extension-abi crates/sdkwork-api-extension-host crates/sdkwork-api-ext-provider-native-mock crates/sdkwork-api-app-gateway crates/sdkwork-api-interface-http
git commit -m "feat: add native dynamic streaming runtime"
git push
```
