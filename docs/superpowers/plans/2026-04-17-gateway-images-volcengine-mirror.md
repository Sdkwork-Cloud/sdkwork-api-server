# Gateway Images Volcengine Mirror Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish `images.volcengine` on Volcengine Ark's official image generation path without adding wrapper routes.

**Architecture:** Add a provider-specific Volcengine image relay for both stateful and stateless routers, expose the same official path in OpenAPI, and keep provider selection driven by `model + mirror_protocol_identity`.

**Tech Stack:** Rust, Axum, utoipa OpenAPI generation, gateway billing/routing store helpers

---

### Task 1: Add failing tests and OpenAPI assertions

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/images_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`

- [ ] **Step 1: Write the failing test**

Add tests that expect:

- stateless relay on `POST /api/v3/images/generations`
- stateful provider selection by model for the same path
- `images.volcengine` OpenAPI tag publication
- no invented wrapper paths such as `/images/volcengine/generations`

- [ ] **Step 2: Run test to verify it fails**

Run:

- `cargo test -p sdkwork-api-interface-http --test images_route volcengine -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --nocapture`

Expected: FAIL because `images.volcengine` is not yet wired.

### Task 2: Implement handlers and route wiring

**Files:**
- Create: `crates/sdkwork-api-interface-http/src/inference_handlers/image_volcengine.rs`
- Create: `crates/sdkwork-api-interface-http/src/inference_stateless_handlers/image_volcengine.rs`
- Modify: `crates/sdkwork-api-interface-http/src/inference_handlers/mod.rs`
- Modify: `crates/sdkwork-api-interface-http/src/inference_stateless_handlers/mod.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_stateful_route_groups/inference_and_storage.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_stateless_route_groups/inference_and_storage.rs`

- [ ] **Step 3: Write minimal implementation**

Implement:

- create relay handler on `POST /api/v3/images/generations`
- stateful provider selection by request model and `volcengine` mirror identity
- stateless relay gated on upstream identity `volcengine`
- billing/usage on route key `images.volcengine.generate`

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-interface-http --test images_route volcengine -- --nocapture`

Expected: PASS

### Task 3: Publish OpenAPI and docs

**Files:**
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_images_volcengine.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi.rs`
- Modify: `docs/api-reference/gateway-api.md`
- Modify: `docs/reference/api-compatibility.md`
- Modify: `docs/zh/api-reference/gateway-api.md`
- Modify: `docs/zh/reference/api-compatibility.md`

- [ ] **Step 5: Publish the official path**

Add:

- `images.volcengine` tag
- `POST /api/v3/images/generations`

- [ ] **Step 6: Run OpenAPI verification**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --nocapture`

Expected: PASS with `images.volcengine` active and no wrapper routes added.

### Task 4: Final verification

**Files:**
- Modify only files touched above as needed

- [ ] **Step 7: Format**

Run: `cargo fmt -p sdkwork-api-interface-http`

- [ ] **Step 8: Run focused suites**

Run:

- `cargo test -p sdkwork-api-interface-http --test images_route -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --nocapture`
- `cargo check -p sdkwork-api-interface-http`

Expected: PASS
