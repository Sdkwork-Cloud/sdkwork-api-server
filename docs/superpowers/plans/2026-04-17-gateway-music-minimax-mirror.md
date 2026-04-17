# Gateway Music MiniMax Mirror Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish `music.minimax` on MiniMax's official `/v1/music_generation` and `/v1/lyrics_generation` paths with direct HTTP relay and stateful provider identity enforcement.

**Architecture:** Reuse the existing provider mirror JSON relay and the existing stateful mirror-identity-constrained routing helper. Keep the shared `/v1/music*` contract and the existing `music.suno` contract unchanged while adding a separate MiniMax mirror family with official unique paths.

**Tech Stack:** Rust, Axum, Reqwest, Utoipa, existing gateway routing and admin store abstractions.

---

### Task 1: Lock OpenAPI Regression Surface

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`

- [ ] **Step 1: Write the failing test**

Assert that:

- `music.minimax` tag exists
- `POST /v1/music_generation` and `POST /v1/lyrics_generation` exist
- wrapper paths like `/music/minimax/*`, `/v1/music/minimax/*`, and `/api/v1/music_generation` remain absent

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --nocapture`

Expected: FAIL because `music.minimax` paths and tag are not published yet.

### Task 2: Lock MiniMax Relay Runtime Behavior

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/music_route.rs`

- [ ] **Step 1: Write the failing tests**

Add tests for:

- stateless MiniMax upstream relay on both official endpoints
- stateful MiniMax route ignores a non-MiniMax provider and relays through a MiniMax provider when both exist
- stateful music generation usage records `music_seconds` from the provider response when available

- [ ] **Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-interface-http --test music_route stateless_music_minimax_routes_relay_to_official_paths -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test music_route stateful_music_minimax_routes_use_minimax_provider_identity -- --nocapture`

Expected: FAIL because MiniMax routes are not registered yet.

### Task 3: Add MiniMax Handlers

**Files:**
- Create: `crates/sdkwork-api-interface-http/src/inference_stateless_handlers/music_minimax.rs`
- Create: `crates/sdkwork-api-interface-http/src/inference_handlers/music_minimax.rs`
- Modify: `crates/sdkwork-api-interface-http/src/inference_stateless_handlers/mod.rs`
- Modify: `crates/sdkwork-api-interface-http/src/inference_handlers/mod.rs`

- [ ] **Step 1: Add stateless handlers**

Relay only when `upstream.mirror_protocol_identity() == "minimax"`.

- [ ] **Step 2: Add stateful handlers**

Use the existing identity-constrained planned execution helper with route keys:

- `music.minimax.generate`
- `music.minimax.lyrics`

- [ ] **Step 3: Record usage**

For `music.minimax.generate`, record usage with `music_seconds` derived from `extra_info.music_duration` when present. For `music.minimax.lyrics`, record lightweight usage without inferred duration.

### Task 4: Wire MiniMax Routes and OpenAPI

**Files:**
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_music_minimax.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_stateless_route_groups/inference_and_storage.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_stateful_route_groups/inference_and_storage.rs`

- [ ] **Step 1: Add OpenAPI path declarations**

Publish the two official MiniMax endpoints with tag `music.minimax` and `serde_json::Value` request/response bodies.

- [ ] **Step 2: Wire routers**

Expose the MiniMax routes on the exact official provider paths.

### Task 5: Update Product Documentation

**Files:**
- Modify: `docs/api-reference/gateway-api.md`
- Modify: `docs/reference/api-compatibility.md`
- Modify: `docs/zh/api-reference/gateway-api.md`
- Modify: `docs/zh/reference/api-compatibility.md`

- [ ] **Step 1: Replace “reserved-only” wording for `music.minimax`**

Document that `music.minimax` is now active and list the official paths.

- [ ] **Step 2: Keep other image/video/music provider families as future governance names**

Do not accidentally mark `music.google` or the remaining image/video provider families as implemented.

### Task 6: Verification

**Files:**
- No code changes expected

- [ ] **Step 1: Run focused HTTP tests**

Run:

- `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test music_route -- --nocapture`

- [ ] **Step 2: Run package checks**

Run:

- `cargo check -p sdkwork-api-app-gateway -p sdkwork-api-interface-http`

- [ ] **Step 3: Run broader verification if focused checks pass**

Run:

- `cargo test -p sdkwork-api-interface-http --test stateless_upstream_protocol -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test stateless_runtime -- --nocapture`
