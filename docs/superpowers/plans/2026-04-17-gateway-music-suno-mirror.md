# Gateway Music Suno Mirror Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish the first provider-specific `music.suno` mirror family on official Suno paths with direct HTTP relay and stateful provider identity enforcement.

**Architecture:** Add generic provider-official mirror relay helpers in `sdkwork-api-interface-http`, add an identity-constrained planned execution selector in `sdkwork-api-app-gateway`, then wire Suno OpenAPI and router endpoints on the exact official Suno paths. Keep the shared `/v1/music*` contract unchanged.

**Tech Stack:** Rust, Axum, Reqwest, Utoipa, existing gateway routing and admin store abstractions.

---

### Task 1: Lock OpenAPI Regression Surface

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`

- [ ] **Step 1: Write the failing test**

Assert that:

- `music.suno` tag exists
- `/api/v1/generate`, `/api/v1/generate/record-info`, `/api/v1/lyrics`, and `/api/v1/lyrics/record-info` exist
- wrapper paths like `/music/suno/*` and `/v1/music/suno/*` remain absent

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --nocapture`

Expected: FAIL because `music.suno` paths/tags are not published yet.

### Task 2: Lock Suno Relay Runtime Behavior

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/music_route.rs`

- [ ] **Step 1: Write the failing tests**

Add tests for:

- stateless Suno upstream relays all four official endpoints and sends `Authorization: Bearer <api_key>`
- stateful Suno route ignores a non-Suno provider and relays through a Suno provider when both exist

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-api-interface-http --test music_route -- --nocapture`

Expected: FAIL because Suno routes are not registered and identity-constrained selection does not exist yet.

### Task 3: Add Identity-Constrained Planned Execution

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/gateway_routing.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`

- [ ] **Step 1: Implement a helper that selects planned execution for a required `mirror_protocol_identity`**

Build a new public helper that:

- simulates a routing decision for `capability + route_key`
- iterates ranked candidates in routing order
- filters to providers whose `mirror_protocol_identity()` matches the requested identity
- resolves execution descriptors using existing account/provider resolution
- returns `PlannedExecutionProviderContext`

- [ ] **Step 2: Add or extend tests if needed through interface coverage**

Use `music_route.rs` stateful coverage as the regression proof instead of adding a separate lower-level unit test unless the helper is otherwise hard to validate.

### Task 4: Add Generic Provider-Official HTTP Relay Helpers

**Files:**
- Create or modify: `crates/sdkwork-api-interface-http/src/gateway_provider_mirror_relay.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`

- [ ] **Step 1: Implement transport helpers**

Support:

- request method passthrough
- full path/query passthrough
- body-bytes passthrough
- outbound bearer auth injection
- response status/header/body passthrough

- [ ] **Step 2: Expose helpers to Suno handlers**

Keep the helpers generic so future slices like `images.kling` or `video.google-veo` can reuse them.

### Task 5: Wire Suno Routes and OpenAPI

**Files:**
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_music_suno.rs`
- Create or modify: `crates/sdkwork-api-interface-http/src/provider_mirror_handlers/music_suno.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_routes.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_stateless_route_groups/inference_and_storage.rs`
- Modify: `crates/sdkwork-api-interface-http/src/inference_handlers/mod.rs`
- Modify: `crates/sdkwork-api-interface-http/src/inference_stateless_handlers/mod.rs`

- [ ] **Step 1: Add OpenAPI path declarations**

Publish the four official Suno endpoints with tag `music.suno` and `serde_json::Value` request/response bodies.

- [ ] **Step 2: Add stateless handlers**

Relay only when `upstream.mirror_protocol_identity() == "suno"`.

- [ ] **Step 3: Add stateful handlers**

Use the new identity-constrained planned execution helper with route keys:

- `music.suno.generate`
- `music.suno.generate.record-info`
- `music.suno.lyrics`
- `music.suno.lyrics.record-info`

- [ ] **Step 4: Record gateway usage**

Record usage against the provider-specific route keys after successful relay.

### Task 6: Update Product Documentation

**Files:**
- Modify: `docs/api-reference/gateway-api.md`
- Modify: `docs/reference/api-compatibility.md`
- Modify: `docs/zh/api-reference/gateway-api.md`
- Modify: `docs/zh/reference/api-compatibility.md`

- [ ] **Step 1: Replace “reserved-only” wording for `music.suno`**

Document that `music.suno` is now active and list the official paths.

- [ ] **Step 2: Keep other image/video/music provider families as future governance names**

Do not accidentally mark other reserved families as implemented.

### Task 7: Verification

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
