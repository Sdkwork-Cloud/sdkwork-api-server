# Gateway Music Google Mirror Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish `music.google` on Google Vertex AI's official Lyria predict path without adding wrapper routes.

**Architecture:** Reuse the existing provider-mirror relay, add Google music stateful/stateless handlers, and replace the current Veo-only Google wildcard route entry with a shared Google Vertex models dispatcher that can route both Veo and music actions. Keep OpenAPI truthful by publishing the official music path separately under `music.google`.

**Tech Stack:** Rust, Axum, Reqwest, Utoipa, existing gateway routing and billing helpers.

---

### Task 1: Lock OpenAPI Regression Surface

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`

- [ ] **Step 1: Write the failing test**

Assert that:

- `music.google` tag exists
- `POST /v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict` exists
- wrapper paths like `/music/google/predict` remain absent

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --nocapture`

Expected: FAIL because `music.google` is not yet published.

### Task 2: Lock Google Music Relay Runtime Behavior

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/music_route.rs`

- [ ] **Step 1: Write the failing tests**

Add tests for:

- stateless relay to the official Google predict path with bearer auth passthrough semantics
- stateful routing that ignores a generic provider and selects the Google music mirror provider

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-api-interface-http --test music_route -- --nocapture`

Expected: FAIL because the Google music route is not yet implemented.

### Task 3: Implement Google Music Dispatcher and Handlers

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/gateway_stateful_route_groups/inference_and_storage.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_stateless_route_groups/inference_and_storage.rs`
- Modify: `crates/sdkwork-api-interface-http/src/inference_handlers/mod.rs`
- Modify: `crates/sdkwork-api-interface-http/src/inference_stateless_handlers/mod.rs`
- Create: `crates/sdkwork-api-interface-http/src/inference_handlers/music_google.rs`
- Create: `crates/sdkwork-api-interface-http/src/inference_stateless_handlers/music_google.rs`
- Modify: existing Google Vertex wildcard dispatcher implementation so it can route both Veo and music actions

- [ ] **Step 1: Add a shared Google action parser**

Recognize:

- `:predictLongRunning`
- `:fetchPredictOperation`
- `:predict`

- [ ] **Step 2: Add Google music stateful handling**

Use capability `music`, route key `music.google.predict`, and mirror identity `google`.

- [ ] **Step 3: Add Google music stateless handling**

Relay only when `upstream.mirror_protocol_identity() == "google"`.

### Task 4: Publish OpenAPI and Docs

**Files:**
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_music_google.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi.rs`
- Modify: `docs/api-reference/gateway-api.md`
- Modify: `docs/reference/api-compatibility.md`
- Modify: `docs/zh/api-reference/gateway-api.md`
- Modify: `docs/zh/reference/api-compatibility.md`

- [ ] **Step 1: Add the official Google music path**

Publish `POST /v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict` under tag `music.google`.

- [ ] **Step 2: Update compatibility docs**

Describe `music.google` as active and keep still-missing families reserved only.

### Task 5: Verification

**Files:**
- No code changes expected

- [ ] **Step 1: Run focused tests**

Run:

- `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test music_route -- --nocapture`

- [ ] **Step 2: Run package checks**

Run:

- `cargo check -p sdkwork-api-interface-http`
