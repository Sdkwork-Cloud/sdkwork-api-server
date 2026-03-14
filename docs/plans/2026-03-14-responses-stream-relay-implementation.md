# Responses Stream Relay Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add real `/v1/responses` SSE relay support through provider adapters, gateway dispatch, and HTTP routes.

**Architecture:** Add a dedicated `ResponsesStream` provider request variant, adapt OpenAI-compatible upstreams into the existing provider-owned stream abstraction, and route `CreateResponseRequest.stream == true` through explicit stream code paths in both the gateway and extension host.

**Tech Stack:** Rust, Axum, reqwest, tokio, futures-util, serde_json

---

### Task 1: Add failing tests for `/v1/responses` streaming

**Files:**
- Modify: `crates/sdkwork-api-provider-openai/tests/http_execution.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/responses_route.rs`
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- the OpenAI-compatible provider adapter can stream `/v1/responses`
- the stateful HTTP route relays upstream SSE for `/v1/responses`
- the extension-host request mapping marks response streams as `expects_stream = true`

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-provider-openai adapter_posts_streaming_responses_to_openai_compatible_upstream -- --exact`
- `cargo test -p sdkwork-api-interface-http --test responses_route stateful_responses_route_relays_stream_to_openai_compatible_provider -- --exact`

Expected: FAIL because provider-core has no `ResponsesStream` request variant, the gateway does not branch to a stream relay path, and the HTTP route always serializes JSON.

### Task 2: Add explicit response stream execution to the provider boundary

**Files:**
- Modify: `crates/sdkwork-api-provider-core/src/lib.rs`
- Modify: `crates/sdkwork-api-provider-openai/src/lib.rs`
- Modify: `crates/sdkwork-api-provider-openrouter/src/lib.rs`
- Modify: `crates/sdkwork-api-provider-ollama/src/lib.rs`

**Step 1: Introduce `ProviderRequest::ResponsesStream`**

Mirror the existing chat stream split with an explicit stream request variant.

**Step 2: Implement upstream `/v1/responses` streaming**

Add `responses_stream` methods in the OpenAI-compatible providers and return `ProviderOutput::Stream`.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-provider-openai -q`

Expected: PASS

### Task 3: Wire gateway and extension host response streams

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`

**Step 1: Add a dedicated gateway relay path**

Implement `relay_response_stream_from_store(...) -> Result<Option<ProviderStreamOutput>>`.

**Step 2: Extend extension-host invocation mapping**

Map `ResponsesStream` to `responses.create` with `expects_stream = true`.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-extension-host -q`
- `cargo test -p sdkwork-api-app-gateway -q`

Expected: PASS

### Task 4: Implement HTTP route branching and fallback SSE

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/responses_route.rs`
- Modify: `README.md`

**Step 1: Route stateful response streams**

When `request.stream == true`, relay upstream SSE via the new gateway function.

**Step 2: Add deterministic local fallback**

Make both stateless and stateful local fallback paths emit a minimal SSE sequence for `/v1/responses`.

**Step 3: Update docs**

Document that `/v1/responses` now has stream relay parity alongside `/v1/chat/completions`.

**Step 4: Run focused tests**

Run:

- `cargo test -p sdkwork-api-interface-http --test responses_route -q`

Expected: PASS

### Task 5: Run verification and commit

**Files:**
- Modify all implementation and documentation files above

**Step 1: Run verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `$env:CARGO_BUILD_JOBS='1'; cargo clippy --no-deps -p sdkwork-api-provider-core -p sdkwork-api-provider-openai -p sdkwork-api-extension-host -p sdkwork-api-app-gateway -p sdkwork-api-interface-http --all-targets -- -D warnings`
- `$env:CARGO_BUILD_JOBS='1'; cargo test --workspace -q -j 1`

Expected: PASS

**Step 2: Commit**

```bash
git add README.md docs/plans/2026-03-14-responses-stream-relay-design.md docs/plans/2026-03-14-responses-stream-relay-implementation.md crates/sdkwork-api-provider-core crates/sdkwork-api-provider-openai crates/sdkwork-api-provider-openrouter crates/sdkwork-api-provider-ollama crates/sdkwork-api-extension-host crates/sdkwork-api-app-gateway crates/sdkwork-api-interface-http
git commit -m "feat: add responses stream relay"
git push
```
