# Gateway Audio And Music Mirror Governance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

> **Superseded status:** This plan reflects an earlier governance stage. The current implemented/public contract still uses `audio.openai` and `music.openai` for the shared `/v1/audio/*` and `/v1/music*` surfaces, while `music.suno`, `music.google`, and `music.minimax` are now active provider-specific mirror families rather than reserved future names.

**Goal:** Publish `audio.openai` and `music.openai` as the active shared media mirror families while keeping `/v1/audio/*` and `/v1/music*` unchanged.

**Architecture:** Treat audio and music governance as a public contract hardening slice, not a runtime routing expansion. Update OpenAPI tags and `operationId` values first through failing regression coverage, then align docs so the generated schema and narrative docs describe the same active mirror families and reserved future groups.

**Tech Stack:** Rust, axum, utoipa, cargo test, markdown docs

---

### Task 1: Add failing OpenAPI governance coverage for audio and music

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`
- Test: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`

- [ ] **Step 1: Write failing assertions for audio and music governance**

Add assertions that require:

- `audio.openai` and `music.openai` tags
- absence of generic `audio` and `music` tags
- absence of reserved music tags
- stable `operationId` values for all `/v1/audio/*` and `/v1/music*` operations
- negative path assertions for fake wrapper prefixes

- [ ] **Step 2: Run the focused OpenAPI regression to verify it fails**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --nocapture`

Expected: FAIL because the current schema still publishes `audio` and `music` tags and is missing the new `operationId` values.

- [ ] **Step 3: Commit the failing-test checkpoint**

```bash
git add crates/sdkwork-api-interface-http/tests/openapi_route.rs
git commit -m "test: cover audio and music mirror governance"
```

### Task 2: Implement the OpenAPI taxonomy and operation IDs

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_media.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_music.rs`
- Test: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`

- [ ] **Step 1: Update gateway tags in the OpenAPI root**

Change the published tag names and descriptions from generic `audio` and `music` to:

- `audio.openai`
- `music.openai`

Keep the public HTTP paths unchanged.

- [ ] **Step 2: Add stable audio `operationId` values and tags**

Update all `/v1/audio/*` operations in `gateway_openapi_paths_media.rs` to use:

- `audio.openai`
- the `audio_openai_*` `operationId` pattern

- [ ] **Step 3: Add stable music `operationId` values and tags**

Update all `/v1/music*` operations in `gateway_openapi_paths_music.rs` to use:

- `music.openai`
- the `music_openai_*` `operationId` pattern

- [ ] **Step 4: Re-run the focused OpenAPI regression**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit the implementation checkpoint**

```bash
git add crates/sdkwork-api-interface-http/src/gateway_openapi.rs crates/sdkwork-api-interface-http/src/gateway_openapi_paths_media.rs crates/sdkwork-api-interface-http/src/gateway_openapi_paths_music.rs crates/sdkwork-api-interface-http/tests/openapi_route.rs
git commit -m "refactor: govern gateway audio and music mirror openapi"
```

### Task 3: Align the English public docs with the implemented contract

**Files:**
- Modify: `docs/api-reference/gateway-api.md`
- Modify: `docs/reference/api-compatibility.md`

- [ ] **Step 1: Update the gateway API reference**

Describe:

- `audio.openai` as the active shared audio mirror family
- `music.openai` as the active shared music mirror family
- reserved `music.suno`, `music.google`, and `music.minimax` families as future-only governance names
- the absence of wrapper prefixes

- [ ] **Step 2: Update the compatibility reference**

Make the compatibility page reflect the same active tags and reserved music families.

- [ ] **Step 3: Verify docs text with the OpenAPI contract**

Read the updated files and confirm the wording matches the implemented tags and path rules.

- [ ] **Step 4: Commit the docs checkpoint**

```bash
git add docs/api-reference/gateway-api.md docs/reference/api-compatibility.md
git commit -m "docs: document gateway audio and music mirror governance"
```

### Task 4: Run the verification matrix before claiming completion

**Files:**
- Verify only

- [ ] **Step 1: Run the full OpenAPI regression suite**

Run: `cargo test -p sdkwork-api-interface-http --test openapi_route -- --nocapture`

Expected: PASS

- [ ] **Step 2: Run the music route regression suite**

Run: `cargo test -p sdkwork-api-interface-http --test music_route -- --nocapture`

Expected: PASS

- [ ] **Step 3: Run a package-level compile check**

Run: `cargo check -p sdkwork-api-interface-http`

Expected: PASS

- [ ] **Step 4: Run the Rust verification helper for interface OpenAPI**

Run: `node scripts/check-rust-verification-matrix.mjs --group interface-openapi`

Expected: PASS

- [ ] **Step 5: Capture remaining follow-ups explicitly**

If everything above passes, record the remaining deferred items separately:

- provider-specific image groups remain reserved only
- provider-specific video groups remain reserved only
- Chinese docs still need sync

- [ ] **Step 6: Commit the verified slice**

```bash
git add docs/superpowers/specs/2026-04-16-gateway-audio-music-mirror-governance-design.md docs/superpowers/plans/2026-04-16-gateway-audio-music-mirror-governance.md docs/api-reference/gateway-api.md docs/reference/api-compatibility.md crates/sdkwork-api-interface-http/src/gateway_openapi.rs crates/sdkwork-api-interface-http/src/gateway_openapi_paths_media.rs crates/sdkwork-api-interface-http/src/gateway_openapi_paths_music.rs crates/sdkwork-api-interface-http/tests/openapi_route.rs
git commit -m "feat: align gateway audio and music mirror governance"
```
