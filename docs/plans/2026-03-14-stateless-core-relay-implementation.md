# Stateless Core Relay Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an explicit stateless gateway runtime contract and enable optional upstream relay for the core bootstrap APIs in stateless mode.

**Architecture:** Extend the HTTP interface crate with a typed stateless config and router state, then reuse the existing provider adapter boundary to relay core API requests directly when a stateless upstream is configured. Keep local emulation as the fallback when no stateless upstream exists.

**Tech Stack:** Rust, Axum, Reqwest, existing SDKWork provider adapter crates

---

### Task 1: Define stateless runtime contract behavior with failing tests

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/models_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/chat_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/chat_stream_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/responses_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/embeddings_route.rs`
- Create: `crates/sdkwork-api-interface-http/tests/stateless_runtime.rs`

**Step 1: Write the failing tests**

Add tests that require:

- a default `StatelessGatewayConfig`
- a custom stateless config constructor
- stateless `/v1/models` relay to a mock upstream when configured
- stateless `/v1/chat/completions` JSON and SSE relay when configured
- stateless `/v1/responses` JSON and SSE relay when configured
- stateless `/v1/embeddings` relay when configured

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-interface-http --test stateless_runtime -q`
- `cargo test -p sdkwork-api-interface-http --test models_route -q`
- `cargo test -p sdkwork-api-interface-http --test chat_route -q`
- `cargo test -p sdkwork-api-interface-http --test chat_stream_route -q`
- `cargo test -p sdkwork-api-interface-http --test responses_route -q`
- `cargo test -p sdkwork-api-interface-http --test embeddings_route -q`

Expected: FAIL because stateless runtime config and stateless upstream relay do not exist yet.

### Task 2: Add provider support for stateless model relay

**Files:**
- Modify: `crates/sdkwork-api-provider-core/src/lib.rs`
- Modify: `crates/sdkwork-api-provider-openai/src/lib.rs`
- Modify: `crates/sdkwork-api-provider-openrouter/src/lib.rs`
- Modify: `crates/sdkwork-api-provider-ollama/src/lib.rs`

**Step 1: Write the failing adapter behavior test**

Extend or add focused provider tests so the OpenAI-compatible adapter must support:

- model list
- model retrieve

**Step 2: Run test to verify it fails**

Run:

- `cargo test -p sdkwork-api-provider-openai -q`

Expected: FAIL because provider request variants or adapter methods do not yet exist.

**Step 3: Write minimal implementation**

Add:

- `ProviderRequest::ModelsList`
- `ProviderRequest::ModelsRetrieve`
- OpenAI-compatible adapter methods and execution wiring
- delegating methods in the OpenRouter and Ollama wrappers

**Step 4: Run test to verify it passes**

Run:

- `cargo test -p sdkwork-api-provider-openai -q`

Expected: PASS

### Task 3: Implement stateless config and core relay in the gateway router

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`

**Step 1: Add typed stateless state**

Introduce:

- `StatelessGatewayConfig`
- `StatelessGatewayContext`
- optional stateless upstream runtime settings
- new router constructor for custom stateless config

**Step 2: Replace hardcoded literals**

Update stateless handlers so they derive tenant and project from the stateless context instead of hardcoded string literals.

**Step 3: Add core stateless relay**

For the bootstrap APIs only, try stateless upstream execution first:

- models list and retrieve
- chat completions
- completions
- responses
- embeddings

Preserve existing local fallback when no stateless upstream is configured.

**Step 4: Run focused tests**

Run:

- `cargo test -p sdkwork-api-interface-http --test stateless_runtime -q`
- `cargo test -p sdkwork-api-interface-http --test models_route -q`
- `cargo test -p sdkwork-api-interface-http --test chat_route -q`
- `cargo test -p sdkwork-api-interface-http --test chat_stream_route -q`
- `cargo test -p sdkwork-api-interface-http --test responses_route -q`
- `cargo test -p sdkwork-api-interface-http --test embeddings_route -q`

Expected: PASS

### Task 4: Document stateless runtime relay behavior

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/api/compatibility-matrix.md`

**Step 1: Update docs**

Document:

- the explicit stateless runtime context
- the new stateless custom router constructor
- optional stateless upstream relay for the bootstrap APIs
- unchanged local fallback behavior when no upstream is configured

**Step 2: Verify doc accuracy**

Ensure docs describe current behavior, not aspirational future routing.

### Task 5: Run full verification and commit

**Files:**
- Modify: repository worktree from previous tasks

**Step 1: Run verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`
- `pnpm --dir console build`
- `$env:CARGO_BUILD_JOBS='1'; cargo clippy --workspace --all-targets -- -D warnings`

Expected: all commands exit `0`

**Step 2: Commit**

```bash
git add docs/plans/2026-03-14-stateless-core-relay-design.md docs/plans/2026-03-14-stateless-core-relay-implementation.md crates/sdkwork-api-interface-http crates/sdkwork-api-provider-core crates/sdkwork-api-provider-openai crates/sdkwork-api-provider-openrouter crates/sdkwork-api-provider-ollama README.md README.zh-CN.md docs/architecture/runtime-modes.md docs/api/compatibility-matrix.md Cargo.lock Cargo.toml
git commit -m "feat: add stateless core relay runtime"
```
