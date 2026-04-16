# Gateway Images Mirror Governance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish the gateway image OpenAPI contract as an OpenAI mirror surface under `images.openai`, reserve future provider families without exposing them publicly, and add regression guardrails that prevent wrapper-path and taxonomy drift.

**Architecture:** Keep the live `/v1/images/*` router behavior unchanged and limit this slice to OpenAPI metadata, docs, and regression tests. The shared image contract remains the official OpenAI image protocol, while provider-specific image family names stay reserved in docs and design governance until real mirror routes exist.

**Tech Stack:** Rust, Axum, utoipa, serde_json, Markdown docs, existing gateway image route tests

---

## File Map

### Gateway OpenAPI Metadata

- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi.rs`
  - Rename the published image tag to `images.openai`.
  - Keep reserved image provider names out of the generated `tags(...)` list.

- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_media.rs`
  - Retag the three `/v1/images/*` operations to `images.openai`.
  - Stabilize their `operation_id` values as `images_openai_*`.

### Regression Tests

- Modify: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`
  - Add failing tests for the new image tag taxonomy.
  - Assert reserved provider tags stay absent.
  - Assert image `operationId` values and wrapper-path absence.

- Keep unchanged: `crates/sdkwork-api-interface-http/tests/images_route.rs`
  - This file already proves runtime behavior for `/v1/images/*`.
  - It should remain green without semantic changes.

### Documentation

- Modify: `docs/api-reference/gateway-api.md`
  - Describe `images.openai` as the current active image mirror family.

- Modify: `docs/reference/api-compatibility.md`
  - Describe provider-specific image families as reserved future mirror groups, not active public contracts.

## Task 1: Lock The New Image OpenAPI Contract In Tests

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`

- [ ] **Step 1: Write the failing test for the new image tag taxonomy**

Add assertions like:

```rust
assert!(tags.iter().any(|tag| tag["name"] == "images.openai"));
assert!(!tags.iter().any(|tag| tag["name"] == "images"));
assert!(!tags.iter().any(|tag| tag["name"] == "images.nanobanana"));
assert!(!tags.iter().any(|tag| tag["name"] == "images.midjourney"));
assert!(!tags.iter().any(|tag| tag["name"] == "images.volcengine"));
assert!(!tags.iter().any(|tag| tag["name"] == "images.aliyun"));
assert!(!tags.iter().any(|tag| tag["name"] == "images.kling"));
```

- [ ] **Step 2: Write the failing test for image operation tags and operation IDs**

Add assertions like:

```rust
assert_eq!(json["paths"]["/v1/images/generations"]["post"]["tags"][0], "images.openai");
assert_eq!(
    json["paths"]["/v1/images/generations"]["post"]["operationId"],
    "images_openai_generations_create"
);
```

Repeat for `/v1/images/edits` and `/v1/images/variations`.

- [ ] **Step 3: Write the failing test for fake image wrapper paths**

Add assertions like:

```rust
assert!(json["paths"]["/images/openai/generations"].is_null());
assert!(json["paths"]["/images/nanobanana/generations"].is_null());
assert!(json["paths"]["/v1/images/nanobanana/generations"].is_null());
assert!(json["paths"]["/v1/images/midjourney/generations"].is_null());
```

- [ ] **Step 4: Run the focused OpenAPI regression and verify it fails**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --exact --nocapture`

Expected: FAIL because the current document still publishes `images` and does not yet publish `images.openai` or the new `operationId` values.

- [ ] **Step 5: Commit the failing-test checkpoint if the team wants a red-stage commit**

```bash
git add crates/sdkwork-api-interface-http/tests/openapi_route.rs
git commit -m "test: define gateway images mirror governance expectations"
```

## Task 2: Retag The Active Image Mirror Operations

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_media.rs`

- [ ] **Step 1: Change the published image tag name in `gateway_openapi.rs`**

Replace:

```rust
(name = "images", description = "Image generation, edit, and variation routes.")
```

With:

```rust
(name = "images.openai", description = "Official OpenAI image mirror routes.")
```

- [ ] **Step 2: Retag all `/v1/images/*` OpenAPI path stubs**

Use:

```rust
#[utoipa::path(
    post,
    path = "/v1/images/generations",
    operation_id = "images_openai_generations_create",
    tag = "images.openai",
    /* existing request and response metadata */
)]
```

Repeat with:

- `images_openai_edits_create`
- `images_openai_variations_create`

- [ ] **Step 3: Keep audio routes unchanged**

Do not change the existing audio tags or route behavior in `gateway_openapi_paths_media.rs`. This slice is only about image governance.

- [ ] **Step 4: Re-run the focused OpenAPI regression and verify it passes**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --exact --nocapture`

Expected: PASS for the new image taxonomy and wrapper-path assertions.

- [ ] **Step 5: Commit the OpenAPI contract update**

```bash
git add crates/sdkwork-api-interface-http/src/gateway_openapi.rs crates/sdkwork-api-interface-http/src/gateway_openapi_paths_media.rs crates/sdkwork-api-interface-http/tests/openapi_route.rs
git commit -m "refactor: govern gateway image mirror openapi contract"
```

## Task 3: Update Public Docs To Match The Active Contract

**Files:**
- Modify: `docs/api-reference/gateway-api.md`
- Modify: `docs/reference/api-compatibility.md`

- [ ] **Step 1: Update the gateway API reference image family wording**

Document that:

- `images.openai` is the current active image mirror family
- `/v1/images/generations`, `/v1/images/edits`, and `/v1/images/variations` remain the official public paths
- provider routing may vary behind the shared OpenAI image contract

- [ ] **Step 2: Update the compatibility reference**

Document that:

- OpenAI image routes are the shared public contract where applicable
- provider-specific image families such as `images.nanobanana` and `images.midjourney` are reserved future mirror groups
- reserved future groups are not yet active public routes

- [ ] **Step 3: Attempt docs verification**

Run: `pnpm -C docs build`

Expected: PASS if `docs/node_modules` is available. If the environment still lacks `vitepress`, record the failure clearly and continue with code/test verification.

- [ ] **Step 4: Commit the documentation refresh**

```bash
git add docs/api-reference/gateway-api.md docs/reference/api-compatibility.md
git commit -m "docs: document gateway image mirror governance"
```

## Task 4: Run Focused Verification

**Files:**
- Modify: touched files only if verification reveals drift

- [ ] **Step 1: Re-run the full OpenAPI contract regression**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route -- --nocapture`

Expected: PASS.

- [ ] **Step 2: Re-run the runtime image regression suite**

Run: `cargo test -p sdkwork-api-interface-http --test images_route -- --nocapture`

Expected: PASS, proving no runtime regression while changing only contract metadata.

- [ ] **Step 3: Re-run compile and verification matrix checks**

Run:

```bash
cargo check -p sdkwork-api-interface-http
node scripts/check-rust-verification-matrix.mjs --group interface-openapi
```

Expected:

- `cargo check -p sdkwork-api-interface-http` exits `0`
- the verification-matrix script exits `0`

- [ ] **Step 4: Review final diff and commit the verification pass**

```bash
git status --short
git diff --stat
git add -A
git commit -m "test: verify gateway image mirror governance"
```
