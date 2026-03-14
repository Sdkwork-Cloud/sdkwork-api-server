# Core Route Hardening Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Harden the core OpenAI-compatible routes so upstream failures return OpenAI-style error envelopes and the gateway avoids rebuilding the extension host on every repeated request.

**Architecture:** Add a small response helper in `sdkwork-api-interface-http` for structured upstream-failure responses on the core endpoints. Add a cache in `sdkwork-api-app-gateway` keyed by effective extension discovery environment values so provider execution can reuse a resolved extension host while preserving correctness when configuration changes.

**Tech Stack:** Rust, Axum, anyhow, serde, OnceLock, Mutex

---

### Task 1: Add failing route compatibility tests

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/models_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/chat_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/responses_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/embeddings_route.rs`

**Step 1: Write the failing tests**

Add route tests showing that upstream relay failures return:

- HTTP `502`
- JSON body with `error.message`
- JSON body with `error.type`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test -p sdkwork-api-interface-http models_route_returns_openai_error_envelope_on_upstream_failure -q
cargo test -p sdkwork-api-interface-http stateless_chat_route_returns_openai_error_envelope_on_upstream_failure -q
cargo test -p sdkwork-api-interface-http stateless_responses_route_returns_openai_error_envelope_on_upstream_failure -q
cargo test -p sdkwork-api-interface-http stateless_embeddings_route_returns_openai_error_envelope_on_upstream_failure -q
```

Expected: failures because the handlers still return plain-text relay errors

### Task 2: Add failing gateway cache tests

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs`

**Step 1: Write the failing test**

Add a test proving repeated runtime execution with a stable extension discovery configuration reuses a cached host instance or reuses cached discovery state instead of rebuilding it from scratch.

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test -p sdkwork-api-app-gateway extension_host_cache_reuses_configured_host_for_stable_policy -q
```

Expected: failure because the current implementation rebuilds the host for each execution

### Task 3: Implement minimal production fixes

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`

**Step 1: Add the structured error response helper**

Use `OpenAiErrorResponse` to produce JSON `502` responses for the scoped core routes.

**Step 2: Add the extension host cache**

Cache the configured extension host by effective discovery-policy environment state.

**Step 3: Run focused tests to verify they pass**

Run:

```powershell
cargo test -p sdkwork-api-interface-http models_route_returns_openai_error_envelope_on_upstream_failure -q
cargo test -p sdkwork-api-interface-http stateless_chat_route_returns_openai_error_envelope_on_upstream_failure -q
cargo test -p sdkwork-api-interface-http stateless_responses_route_returns_openai_error_envelope_on_upstream_failure -q
cargo test -p sdkwork-api-interface-http stateless_embeddings_route_returns_openai_error_envelope_on_upstream_failure -q
cargo test -p sdkwork-api-app-gateway extension_host_cache_reuses_configured_host_for_stable_policy -q
```

Expected: PASS

### Task 4: Re-run broader verification

**Files:**
- Review: interface-http, app-gateway

**Step 1: Run broader validation**

Run:

```powershell
cargo test -p sdkwork-api-interface-http -q
cargo test -p sdkwork-api-app-gateway -q
cargo test --workspace -q -j 1
```

Expected: PASS

### Task 5: Commit the batch

**Files:**
- Include updated tests, production code, and plan docs

**Step 1: Commit**

Run:

```powershell
git add crates/sdkwork-api-interface-http crates/sdkwork-api-app-gateway docs/plans/2026-03-14-core-route-hardening-design.md docs/plans/2026-03-14-core-route-hardening-implementation.md
git commit -m "fix: harden core route relay handling"
```
