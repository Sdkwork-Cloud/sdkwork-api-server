# Gateway Video Mirror Governance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

> **Superseded status:** This plan reflects an earlier governance stage. The current implemented/public contract keeps Sora 2 and Sora 2 Pro on `video.openai`, does not publish `video.sora`, and now publishes `video.kling`, `video.aliyun`, `video.google-veo`, `video.minimax`, `video.vidu`, and `video.volcengine` as active provider-specific mirror families.

**Goal:** Publish the gateway video OpenAPI contract as a mirror surface under `video.openai`, reserve future provider-specific video families without exposing them publicly, and add regression guardrails that prevent wrapper-path and taxonomy drift.

**Architecture:** Keep the live `/v1/videos*` router behavior unchanged and limit this slice to OpenAPI metadata, docs, and regression tests. The shared video contract remains the current public `/v1/videos*` protocol, while provider-specific video family names stay reserved in docs and design governance until real mirror routes exist.

**Tech Stack:** Rust, Axum, utoipa, serde_json, Markdown docs, existing gateway video route tests

---

## File Map

### Gateway OpenAPI Metadata

- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi.rs`
  - Rename the published video tag to `video.openai`.
  - Keep reserved video provider names out of the generated `tags(...)` list.

- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_video.rs`
  - Retag every `/v1/videos*` operation to `video.openai`.
  - Stabilize their `operation_id` values as `video_openai_*`.

### Regression Tests

- Modify: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`
  - Add failing tests for the new video tag taxonomy.
  - Assert reserved provider tags stay absent.
  - Assert video `operationId` values and wrapper-path absence.

- Keep unchanged: `crates/sdkwork-api-interface-http/tests/videos_route/mod.rs`
  - This module already proves runtime behavior for `/v1/videos*`.
  - It should remain green without semantic changes.

### Documentation

- Modify: `docs/api-reference/gateway-api.md`
  - Describe `video.openai` as the current active video mirror family.

- Modify: `docs/reference/api-compatibility.md`
  - Describe provider-specific video families as reserved future mirror groups, not active public contracts.

## Task 1: Lock The New Video OpenAPI Contract In Tests

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`

- [ ] **Step 1: Write the failing test for the new video tag taxonomy**

Add assertions like:

```rust
assert!(tags.iter().any(|tag| tag["name"] == "video.openai"));
assert!(!tags.iter().any(|tag| tag["name"] == "videos"));
assert!(!tags.iter().any(|tag| tag["name"] == "video.sora"));
assert!(!tags.iter().any(|tag| tag["name"] == "video.minimax"));
assert!(!tags.iter().any(|tag| tag["name"] == "video.vidu"));
assert!(!tags.iter().any(|tag| tag["name"] == "video.volcengine"));
assert!(!tags.iter().any(|tag| tag["name"] == "video.google-veo"));
assert!(!tags.iter().any(|tag| tag["name"] == "video.aliyun"));
assert!(!tags.iter().any(|tag| tag["name"] == "video.kling"));
```

- [ ] **Step 2: Write the failing test for video operation tags and operation IDs**

Add assertions like:

```rust
assert_eq!(json["paths"]["/v1/videos"]["get"]["tags"][0], "video.openai");
assert_eq!(json["paths"]["/v1/videos"]["get"]["operationId"], "video_openai_list");
assert_eq!(json["paths"]["/v1/videos"]["post"]["operationId"], "video_openai_create");
```

Repeat for the remaining `/v1/videos*` operations.

- [ ] **Step 3: Write the failing test for fake video wrapper paths**

Add assertions like:

```rust
assert!(json["paths"]["/video/openai/videos"].is_null());
assert!(json["paths"]["/video/sora/videos"].is_null());
assert!(json["paths"]["/v1/videos/sora/create"].is_null());
assert!(json["paths"]["/v1/video/create"].is_null());
```

- [ ] **Step 4: Run the focused OpenAPI regression and verify it fails**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --exact --nocapture`

Expected: FAIL because the current document still publishes `videos` and does not yet publish `video.openai` or the new `operationId` values.

## Task 2: Retag The Active Video Mirror Operations

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_video.rs`

- [ ] **Step 1: Change the published video tag name in `gateway_openapi.rs`**

Replace:

```rust
(name = "videos", description = "Video generation, transforms, and character routes.")
```

With:

```rust
(name = "video.openai", description = "Official shared video mirror routes.")
```

- [ ] **Step 2: Retag all `/v1/videos*` OpenAPI path stubs**

Use patterns like:

```rust
#[utoipa::path(
    get,
    path = "/v1/videos",
    operation_id = "video_openai_list",
    tag = "video.openai",
    /* existing response metadata */
)]
```

Set these `operation_id` values:

- `video_openai_list`
- `video_openai_create`
- `video_openai_get`
- `video_openai_delete`
- `video_openai_content_get`
- `video_openai_remix_create`
- `video_openai_characters_create`
- `video_openai_character_canonical_get`
- `video_openai_edits_create`
- `video_openai_extensions_create`
- `video_openai_characters_list`
- `video_openai_character_get`
- `video_openai_character_update`
- `video_openai_extend_create`

- [ ] **Step 3: Keep runtime behavior unchanged**

Do not change the actual routes in `gateway_routes.rs`, stateful groups, or stateless groups. This slice is only about public contract governance.

- [ ] **Step 4: Re-run the focused OpenAPI regression and verify it passes**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --exact --nocapture`

Expected: PASS for the new video taxonomy and wrapper-path assertions.

- [ ] **Step 5: Commit the OpenAPI contract update**

```bash
git add crates/sdkwork-api-interface-http/src/gateway_openapi.rs crates/sdkwork-api-interface-http/src/gateway_openapi_paths_video.rs crates/sdkwork-api-interface-http/tests/openapi_route.rs
git commit -m "refactor: govern gateway video mirror openapi contract"
```

## Task 3: Update Public Docs To Match The Active Contract

**Files:**
- Modify: `docs/api-reference/gateway-api.md`
- Modify: `docs/reference/api-compatibility.md`

- [ ] **Step 1: Update the gateway API reference video family wording**

Document that:

- `video.openai` is the current active video mirror family
- `/v1/videos*` remains the official public path family
- provider routing may vary behind the shared video contract

- [ ] **Step 2: Update the compatibility reference**

Document that:

- the shared `/v1/videos*` routes are the current public contract
- provider-specific video families such as `video.sora` and `video.minimax` are reserved future mirror groups
- reserved future groups are not yet active public routes

- [ ] **Step 3: Attempt docs verification**

Run: `pnpm -C docs build`

Expected: PASS if `docs/node_modules` is available. If the environment still lacks `vitepress`, record the failure clearly and continue with code/test verification.

- [ ] **Step 4: Commit the documentation refresh**

```bash
git add docs/api-reference/gateway-api.md docs/reference/api-compatibility.md
git commit -m "docs: document gateway video mirror governance"
```

## Task 4: Run Focused Verification

**Files:**
- Modify: touched files only if verification reveals drift

- [ ] **Step 1: Re-run the full OpenAPI contract regression**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route -- --nocapture`

Expected: PASS.

- [ ] **Step 2: Re-run the runtime video regression suite**

Run: `cargo test -p sdkwork-api-interface-http --test videos_route -- --nocapture`

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
git commit -m "test: verify gateway video mirror governance"
```
