# Gateway Video Volcengine Mirror Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish `video.volcengine` on Volcengine's official video task create/get paths without adding wrapper routes.

**Architecture:** Add a provider-specific Volcengine video relay for both stateful and stateless routers, publish the same official paths in OpenAPI, and use billing-event ownership to route task queries back to the provider that created the task.

**Tech Stack:** Rust, Axum, utoipa OpenAPI generation, gateway billing/routing store helpers

---

### Task 1: Add failing route and OpenAPI tests

**Files:**
- Create: `crates/sdkwork-api-interface-http/tests/videos_route/provider_volcengine.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/videos_route/mod.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`

- [ ] **Step 1: Write the failing test**

Add tests that expect:

- stateless routing on `POST /api/v1/contents/generations/tasks`
- stateless routing on `GET /api/v1/contents/generations/tasks/{id}`
- stateful provider selection by model for create
- stateful provider ownership resolution by task `id` for get
- OpenAPI tag/path publication for `video.volcengine`

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-http --test videos_route provider_volcengine -- --nocapture`

Expected: FAIL because `video.volcengine` routes and tag are not yet wired.

### Task 2: Implement Volcengine video handlers and route wiring

**Files:**
- Create: `crates/sdkwork-api-interface-http/src/inference_handlers/video_volcengine.rs`
- Create: `crates/sdkwork-api-interface-http/src/inference_stateless_handlers/video_volcengine.rs`
- Modify: `crates/sdkwork-api-interface-http/src/inference_handlers/mod.rs`
- Modify: `crates/sdkwork-api-interface-http/src/inference_stateless_handlers/mod.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_stateful_route_groups/inference_and_storage.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_stateless_route_groups/inference_and_storage.rs`

- [ ] **Step 3: Write minimal implementation**

Implement:

- create handler that parses JSON, extracts `model`, selects a `volcengine` provider, relays to the official create path, and records task `id`
- get handler that resolves provider ownership from billing events by task `id`, relays to the official get path, and records usage
- stateless handlers that only relay when the configured upstream identity is `volcengine`

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-interface-http --test videos_route provider_volcengine -- --nocapture`

Expected: PASS

### Task 3: Publish OpenAPI and docs

**Files:**
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_video_volcengine.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi.rs`
- Modify: `docs/api-reference/gateway-api.md`
- Modify: `docs/reference/api-compatibility.md`
- Modify: `docs/zh/api-reference/gateway-api.md`
- Modify: `docs/zh/reference/api-compatibility.md`

- [ ] **Step 5: Publish the official paths**

Add:

- `video.volcengine` tag
- `POST /api/v1/contents/generations/tasks`
- `GET /api/v1/contents/generations/tasks/{id}`

- [ ] **Step 6: Run OpenAPI verification**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --nocapture`

Expected: PASS with `video.volcengine` active and no wrapper routes added.

### Task 4: Final verification

**Files:**
- Modify only files touched above as needed

- [ ] **Step 7: Format**

Run: `cargo fmt -p sdkwork-api-interface-http`

- [ ] **Step 8: Run focused suites**

Run:

- `cargo test -p sdkwork-api-interface-http --test videos_route -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --nocapture`
- `cargo check -p sdkwork-api-interface-http`

Expected: PASS
