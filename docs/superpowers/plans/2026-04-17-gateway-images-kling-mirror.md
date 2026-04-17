# Gateway Images Kling Mirror Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish `images.kling` as a provider-specific mirror family on Kling's official submit and task-query paths.

**Architecture:** Reuse the existing provider mirror JSON relay and the stateful mirror-identity-constrained routing helper from the `music.suno` slice. Keep shared OpenAI image routes unchanged while adding a separate Kling mirror group with official paths and decision-log-only accounting.

**Tech Stack:** Rust, Axum, utoipa, serde_json, existing gateway image tests, provider mirror relay helpers

---

### Task 1: Lock the Kling mirror contract with tests

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/images_route.rs`

- [x] **Step 1: Add failing OpenAPI assertions for `images.kling`**
- [x] **Step 2: Add failing stateless runtime assertions for Kling official paths**
- [x] **Step 3: Add failing stateful identity-routing assertions for Kling**
- [x] **Step 4: Run the focused tests and confirm red**

### Task 2: Implement Kling mirror handlers and router wiring

**Files:**
- Create: `crates/sdkwork-api-interface-http/src/inference_stateless_handlers/image_kling.rs`
- Create: `crates/sdkwork-api-interface-http/src/inference_handlers/image_kling.rs`
- Modify: `crates/sdkwork-api-interface-http/src/inference_stateless_handlers/mod.rs`
- Modify: `crates/sdkwork-api-interface-http/src/inference_handlers/mod.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_stateless_route_groups/inference_and_storage.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_stateful_route_groups/inference_and_storage.rs`

- [ ] **Step 1: Add stateless Kling JSON relay handlers**
- [ ] **Step 2: Add stateful Kling mirror handlers with identity-constrained planning**
- [ ] **Step 3: Persist decision logs and keep usage recording disabled**
- [ ] **Step 4: Register the official Kling paths in both routers**

### Task 3: Publish the OpenAPI and docs

**Files:**
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_images_kling.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi.rs`
- Modify: `docs/api-reference/gateway-api.md`
- Modify: `docs/reference/api-compatibility.md`
- Modify: `docs/zh/api-reference/gateway-api.md`
- Modify: `docs/zh/reference/api-compatibility.md`

- [ ] **Step 1: Add `images.kling` OpenAPI stubs**
- [ ] **Step 2: Register the tag and paths in the generated document**
- [ ] **Step 3: Update docs to mark `images.kling` active and keep the other image provider groups reserved**

### Task 4: Verify the slice

**Files:**
- No code changes expected

- [ ] **Step 1: Run `cargo fmt -p sdkwork-api-interface-http`**
- [ ] **Step 2: Run `cargo check -p sdkwork-api-interface-http -p sdkwork-api-app-gateway`**
- [ ] **Step 3: Run the focused Kling tests and confirm green**
- [ ] **Step 4: Re-run the broader `images_route` suite**
