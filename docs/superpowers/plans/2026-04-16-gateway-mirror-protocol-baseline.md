# Gateway Mirror Protocol Baseline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the gateway OpenAPI surface a truthful mirror of the live public router, remove the public `compatibility` bucket, and publish OpenAI/Codex, Claude, and Gemini as first-class mirror protocol groups without changing their official HTTP paths.

**Architecture:** Keep the existing Axum router and execution kernel unchanged, but refactor the OpenAPI document into responsibility-scoped path modules and expand schema coverage for the already-public route families that are currently undocumented. The public HTTP contract remains official-path-first, while OpenAPI grouping moves to stable `capability.protocol` tags such as `code.openai`, `code.claude`, and `code.gemini`.

**Tech Stack:** Rust, Axum, utoipa, serde, serde_json, existing `sdkwork-api-contract-openai` DTOs, Markdown docs, VitePress docs site

---

## File Map

### Gateway OpenAPI Aggregation

- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi.rs`
  - Replace the current mixed module imports and tags with `capability.protocol` groups.
  - Expand `paths(...)` so it fully mirrors the public router inventory already exposed by `gateway_routes.rs`.

### New OpenAPI Path Modules

- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_code_openai.rs`
  - Own `/v1/models`, `/v1/chat/completions`, `/v1/completions`, `/v1/responses`, `/v1/embeddings`, `/v1/moderations`.
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_code_claude.rs`
  - Own `/v1/messages`, `/v1/messages/count_tokens`.
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_code_gemini.rs`
  - Own `/v1beta/models/{tail}` and Gemini mirror descriptions.
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_storage.rs`
  - Own `/v1/containers*`, `/v1/files*`, `/v1/uploads*`, `/v1/vector_stores*`.
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_agents.rs`
  - Own `/v1/assistants*`, `/v1/threads*`, `/v1/conversations*`, `/v1/realtime/sessions`.
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_jobs.rs`
  - Own `/v1/batches*`, `/v1/fine_tuning/*`, `/v1/webhooks*`, `/v1/evals*`.
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_video.rs`
  - Own `/v1/videos*`.
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_music.rs`
  - Own `/v1/music*`.
- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_media.rs`
  - Keep only `/v1/images*` and `/v1/audio/*`.
- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_market_commercial.rs`
  - Keep current market, marketing, and commercial routes unchanged.

### OpenAPI Module Cleanup

- Delete: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_models_chat.rs`
- Delete: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_vector_compat.rs`
- Delete: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_files_batches.rs`
- Delete: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_assistants_threads.rs`

Delete the old mixed modules only after the new modules compile and are wired into `gateway_openapi.rs`.

### Contract DTO Schema Coverage

- Modify: `crates/sdkwork-api-contract-openai/src/containers.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/videos.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/music.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/webhooks.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/evals.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/fine_tuning.rs`

These files already define the public request and response DTOs used by the real router. They need `ToSchema` derives so the OpenAPI document can reference them directly instead of collapsing those families into undocumented JSON placeholders.

### Regression And Contract Tests

- Modify: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`
  - Turn the current route assertions into the parity oracle for tags, path inventory, schema inventory, and wrapper-path absence.

### Documentation

- Modify: `docs/api-reference/gateway-api.md`
- Modify: `docs/zh/api-reference/gateway-api.md`
- Modify: `docs/reference/api-compatibility.md`
- Modify: `docs/zh/reference/api-compatibility.md`
- Modify: `docs/api/compatibility-matrix.md`

These docs must stop describing Claude and Gemini as an implementation-facing `compatibility` bucket and instead document them as first-class mirror protocol families under the gateway.

## Task 1: Lock Code Protocol Tagging And Wrapper-Path Guardrails

**Files:**
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_code_openai.rs`
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_code_claude.rs`
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_code_gemini.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`

- [ ] **Step 1: Write the failing test for the new public tag taxonomy**

Add assertions like:

```rust
assert!(json["tags"].as_array().unwrap().iter().any(|tag| tag["name"] == "code.openai"));
assert!(json["tags"].as_array().unwrap().iter().any(|tag| tag["name"] == "code.claude"));
assert!(json["tags"].as_array().unwrap().iter().any(|tag| tag["name"] == "code.gemini"));
assert!(!json["tags"].as_array().unwrap().iter().any(|tag| tag["name"] == "compatibility"));
```

- [ ] **Step 2: Write the failing test that forbids fake wrapper prefixes**

Add assertions like:

```rust
assert!(json["paths"]["/code"].is_null());
assert!(json["paths"]["/code/chat/completions"].is_null());
assert!(json["paths"]["/claude/messages"].is_null());
assert!(json["paths"]["/gemini/models"].is_null());
```

- [ ] **Step 3: Run the focused regression and verify it fails for missing tags**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --exact --nocapture`

Expected: FAIL because the current OpenAPI document still publishes `compatibility` and does not expose `code.openai`, `code.claude`, or `code.gemini`.

- [ ] **Step 4: Split the current code protocol path stubs into the new module layout**

Move the existing `models/chat/completions/responses/embeddings/moderations` stubs into:

```rust
#[path = "gateway_openapi_paths_code_openai.rs"]
mod paths_code_openai;

#[path = "gateway_openapi_paths_code_claude.rs"]
mod paths_code_claude;

#[path = "gateway_openapi_paths_code_gemini.rs"]
mod paths_code_gemini;
```

Use tags like:

```rust
#[utoipa::path(post, path = "/v1/messages", tag = "code.claude", /* ... */)]
pub(crate) async fn anthropic_messages() {}

#[utoipa::path(post, path = "/v1beta/models/{tail}", tag = "code.gemini", /* ... */)]
pub(crate) async fn gemini_models_compat() {}
```

- [ ] **Step 5: Update `gateway_openapi.rs` tags and path imports**

Replace the old tags with:

```rust
tags(
    (name = "system.sdkwork", description = "Gateway system routes."),
    (name = "code.openai", description = "Official OpenAI and Codex mirror routes."),
    (name = "code.claude", description = "Official Claude mirror routes."),
    (name = "code.gemini", description = "Official Gemini mirror routes.")
)
```

Retain the actual public HTTP paths exactly as they already exist.

- [ ] **Step 6: Re-run the focused regression and verify the new tags are green**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --exact --nocapture`

Expected: still FAIL, but now the remaining failures should be about missing path inventory or missing schemas for the undocumented route families, not about the code protocol taxonomy.

- [ ] **Step 7: Commit the isolated code taxonomy change**

```bash
git add crates/sdkwork-api-interface-http/src/gateway_openapi.rs crates/sdkwork-api-interface-http/src/gateway_openapi_paths_code_openai.rs crates/sdkwork-api-interface-http/src/gateway_openapi_paths_code_claude.rs crates/sdkwork-api-interface-http/src/gateway_openapi_paths_code_gemini.rs crates/sdkwork-api-interface-http/tests/openapi_route.rs
git commit -m "refactor: split gateway code protocol openapi tags"
```

## Task 2: Make Missing Public Route DTO Families Schema-Aware

**Files:**
- Modify: `crates/sdkwork-api-contract-openai/src/containers.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/videos.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/music.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/webhooks.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/evals.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/fine_tuning.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`

- [ ] **Step 1: Extend the failing OpenAPI test with schema expectations for the undocumented families**

Add assertions like:

```rust
assert!(json["components"]["schemas"]["CreateContainerRequest"].is_object());
assert!(json["components"]["schemas"]["VideosResponse"].is_object());
assert!(json["components"]["schemas"]["MusicTracksResponse"].is_object());
assert!(json["components"]["schemas"]["ListWebhooksResponse"].is_object());
assert!(json["components"]["schemas"]["ListEvalsResponse"].is_object());
assert!(json["components"]["schemas"]["FineTuningJobObject"].is_object());
```

- [ ] **Step 2: Run the focused regression and verify the new schema assertions fail**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --exact --nocapture`

Expected: FAIL because those DTO families do not currently derive `ToSchema`.

- [ ] **Step 3: Add `utoipa::ToSchema` derives to the missing request and response DTOs**

Follow the existing contract-crate pattern:

```rust
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateContainerRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ContainerObject {
    pub id: String,
    pub object: &'static str,
    pub name: String,
    pub status: &'static str,
}
```

Do the same for all public DTOs used by:

- containers and container files
- video create, transforms, characters, and delete/list responses
- music create, lyrics, list/delete responses
- webhooks create/update/list/delete
- evals, eval runs, and output items
- fine-tuning jobs, events, checkpoints, permissions, and delete responses

- [ ] **Step 4: Re-run the focused regression and verify failures move to missing path stubs instead of schema absence**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --exact --nocapture`

Expected: FAIL only because the path modules and `paths(...)` list still do not fully cover the real public router.

- [ ] **Step 5: Commit the DTO schema slice**

```bash
git add crates/sdkwork-api-contract-openai/src/containers.rs crates/sdkwork-api-contract-openai/src/videos.rs crates/sdkwork-api-contract-openai/src/music.rs crates/sdkwork-api-contract-openai/src/webhooks.rs crates/sdkwork-api-contract-openai/src/evals.rs crates/sdkwork-api-contract-openai/src/fine_tuning.rs crates/sdkwork-api-interface-http/tests/openapi_route.rs
git commit -m "feat: add openapi schema coverage for extended gateway routes"
```

## Task 3: Publish The Missing Public Route Families In OpenAPI

**Files:**
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_storage.rs`
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_agents.rs`
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_jobs.rs`
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_video.rs`
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_music.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_media.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`
- Delete: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_models_chat.rs`
- Delete: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_vector_compat.rs`
- Delete: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_files_batches.rs`
- Delete: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_assistants_threads.rs`

- [ ] **Step 1: Extend the failing OpenAPI test with the missing live public paths**

Add assertions for the currently under-documented public routes:

```rust
assert!(json["paths"]["/v1/containers"]["get"].is_object());
assert!(json["paths"]["/v1/files/{file_id}/content"]["get"].is_object());
assert!(json["paths"]["/v1/videos"]["post"].is_object());
assert!(json["paths"]["/v1/music/{music_id}/content"]["get"].is_object());
assert!(json["paths"]["/v1/fine_tuning/jobs/{fine_tuning_job_id}/pause"]["post"].is_object());
assert!(json["paths"]["/v1/webhooks/{webhook_id}"]["post"].is_object());
assert!(json["paths"]["/v1/evals/{eval_id}/runs/{run_id}/output_items/{output_item_id}"]["get"].is_object());
```

- [ ] **Step 2: Run the focused regression and verify it fails for the missing paths**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --exact --nocapture`

Expected: FAIL because the current `paths(...)` list still omits those live public routes.

- [ ] **Step 3: Add new `#[utoipa::path]` stubs for the missing route families**

Follow the existing stubs, but use the new tags and real DTO refs:

```rust
#[utoipa::path(
    get,
    path = "/v1/music/{music_id}/content",
    tag = "music.openai",
    params(("music_id" = String, Path, description = "Music track identifier.")),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Music binary content stream."),
        (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
        (status = 404, description = "Requested music was not found.", body = OpenAiErrorResponse)
    )
)]
pub(crate) async fn music_content() {}
```

Use the same style for:

- container create/list/retrieve/delete
- container files create/list/retrieve/delete/content
- file content
- videos create/list/retrieve/delete/content/remix/edits/extensions/extend/characters
- music create/list/retrieve/delete/content/lyrics
- fine-tuning jobs, events, checkpoints, pause, resume, permission create/list/delete
- webhooks list/create/retrieve/update/delete
- evals list/create/retrieve/update/delete
- eval runs list/create/retrieve/delete/cancel/output item list/retrieve

- [ ] **Step 4: Rewire `gateway_openapi.rs` to aggregate the new module layout and retire the mixed modules**

Update:

```rust
mod openapi_paths {
    pub(crate) use super::paths_code_openai::*;
    pub(crate) use super::paths_code_claude::*;
    pub(crate) use super::paths_code_gemini::*;
    pub(crate) use super::paths_storage::*;
    pub(crate) use super::paths_agents::*;
    pub(crate) use super::paths_jobs::*;
    pub(crate) use super::paths_media::*;
    pub(crate) use super::paths_video::*;
    pub(crate) use super::paths_music::*;
    pub(crate) use super::paths_market_commercial::*;
}
```

Also add any missing DTO imports required by the new path stubs.

- [ ] **Step 5: Re-run the focused parity regression and verify it passes**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --exact --nocapture`

Expected: PASS with the new paths, tags, and schema coverage.

- [ ] **Step 6: Commit the OpenAPI parity and module split**

```bash
git add crates/sdkwork-api-interface-http/src/gateway_openapi.rs crates/sdkwork-api-interface-http/src/gateway_openapi_paths_storage.rs crates/sdkwork-api-interface-http/src/gateway_openapi_paths_agents.rs crates/sdkwork-api-interface-http/src/gateway_openapi_paths_jobs.rs crates/sdkwork-api-interface-http/src/gateway_openapi_paths_video.rs crates/sdkwork-api-interface-http/src/gateway_openapi_paths_music.rs crates/sdkwork-api-interface-http/src/gateway_openapi_paths_media.rs crates/sdkwork-api-interface-http/tests/openapi_route.rs
git rm crates/sdkwork-api-interface-http/src/gateway_openapi_paths_models_chat.rs crates/sdkwork-api-interface-http/src/gateway_openapi_paths_vector_compat.rs crates/sdkwork-api-interface-http/src/gateway_openapi_paths_files_batches.rs crates/sdkwork-api-interface-http/src/gateway_openapi_paths_assistants_threads.rs
git commit -m "refactor: align gateway openapi with live public routes"
```

## Task 4: Refresh Gateway Mirror Documentation

**Files:**
- Modify: `docs/api-reference/gateway-api.md`
- Modify: `docs/zh/api-reference/gateway-api.md`
- Modify: `docs/reference/api-compatibility.md`
- Modify: `docs/zh/reference/api-compatibility.md`
- Modify: `docs/api/compatibility-matrix.md`

- [ ] **Step 1: Update the gateway API reference to describe the mirror protocol taxonomy**

Replace wording that implies a generic compatibility bucket with language like:

```md
- `code.openai`: OpenAI and Codex mirror routes on `/v1/*`
- `code.claude`: Claude mirror routes on `/v1/messages*`
- `code.gemini`: Gemini mirror routes on `/v1beta/models/*`
```

- [ ] **Step 2: Update the compatibility reference to explain the public contract rule**

Document:

- official protocol first
- OpenAI standard second when it already exists for the capability
- provider-specific mirror only when no shared standard exists
- no custom wrapper prefixes

- [ ] **Step 3: Update the matrix wording so Claude and Gemini are mirror protocol families, not leftover sidecars**

Keep the execution-truth labels (`translated`, `relay`, etc.) because they still describe runtime behavior, but stop using them as the public taxonomy.

- [ ] **Step 4: Rebuild the docs site**

Run: `pnpm -C docs build`

Expected: PASS with the updated Markdown pages rendered into the docs site.

- [ ] **Step 5: Commit the docs refresh**

```bash
git add docs/api-reference/gateway-api.md docs/zh/api-reference/gateway-api.md docs/reference/api-compatibility.md docs/zh/reference/api-compatibility.md docs/api/compatibility-matrix.md
git commit -m "docs: describe gateway mirror protocol baseline"
```

## Task 5: Run Focused Verification And Final Integration Checks

**Files:**
- Modify: any touched files only if verification exposes drift

- [ ] **Step 1: Run the OpenAPI regression suite**

Run:

```bash
cargo test -p sdkwork-api-interface-http --test openapi_route -- --nocapture
```

Expected: PASS with the full OpenAPI inventory assertions green.

- [ ] **Step 2: Run the mirror protocol regression suite**

Run:

```bash
cargo test -p sdkwork-api-interface-http --test anthropic_messages_route -- --nocapture
cargo test -p sdkwork-api-interface-http --test gemini_generate_content_route -- --nocapture
```

Expected: PASS, proving the Phase 1 tag and OpenAPI work did not regress the existing Claude or Gemini mirror behavior.

- [ ] **Step 3: Run the extended public-route regressions that were newly documented**

Run:

```bash
cargo test -p sdkwork-api-interface-http --test containers_route -- --nocapture
cargo test -p sdkwork-api-interface-http --test videos_route -- --nocapture
cargo test -p sdkwork-api-interface-http --test music_route -- --nocapture
cargo test -p sdkwork-api-interface-http --test fine_tuning_route -- --nocapture
cargo test -p sdkwork-api-interface-http --test webhooks_route -- --nocapture
cargo test -p sdkwork-api-interface-http --test evals_route -- --nocapture
```

Expected: PASS, proving the new OpenAPI inventory matches live route families that already execute successfully.

- [ ] **Step 4: Run formatting, compile, and verification-matrix checks**

Run:

```bash
cargo fmt --all
cargo check -p sdkwork-api-interface-http
node scripts/check-rust-verification-matrix.mjs --group interface-openapi
```

Expected:

- `cargo fmt --all` exits `0`
- `cargo check -p sdkwork-api-interface-http` exits `0`
- the verification-matrix check reports the touched `interface-openapi` lane cleanly

- [ ] **Step 5: Review the final diff and commit the verification pass**

```bash
git status --short
git diff --stat
git add -A
git commit -m "test: verify gateway mirror protocol baseline"
```

- [ ] **Step 6: Record any deferred scope explicitly**

If provider-specific mirror routes for `images.*`, `video.*`, or `music.*` were not part of Phase 1, document them as Phase 2, 3, and 4 follow-ups rather than allowing them to leak into this baseline change.
