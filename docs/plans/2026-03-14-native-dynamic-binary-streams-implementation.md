# Native Dynamic Binary Streams Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `native_dynamic` provider extensions capable of relaying binary stream operations for audio speech, file content, and video content through the gateway.

**Architecture:** Reuse the existing host-owned `ProviderStreamOutput` and callback-based native stream ABI, then mark stream-returning provider requests as `expects_stream = true` so native dynamic plugins can emit bytes plus content type for binary routes.

**Tech Stack:** Rust, Axum, tokio, serde_json, libloading

---

### Task 1: Add failing tests for native dynamic binary stream operations

**Files:**
- Modify: `crates/sdkwork-api-extension-host/tests/native_dynamic_runtime.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/audio_speech_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/files_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/videos_route.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- native dynamic runtime can stream audio speech bytes
- native dynamic runtime can stream file content bytes
- native dynamic runtime can stream video content bytes
- stateful HTTP routes relay those bytes through native dynamic providers

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-extension-host --test native_dynamic_runtime executes_native_dynamic_audio_speech_stream_request -- --exact`
- `cargo test -p sdkwork-api-interface-http --test audio_speech_route stateful_audio_speech_route_relays_to_native_dynamic_provider -- --exact`

Expected: FAIL because host invocation mapping still marks these operations as non-streaming.

### Task 2: Wire stream expectation for binary operations

**Files:**
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`

**Step 1: Mark stream-returning operations correctly**

Set `expects_stream = true` for:

- `audio.speech.create`
- `files.content`
- `videos.content`

**Step 2: Add invocation mapping tests**

Verify the provider invocation contract now flags these operations as streaming.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-extension-host -q`

Expected: PASS after fixture support is added.

### Task 3: Extend the native fixture and manifest capability set

**Files:**
- Modify: `crates/sdkwork-api-ext-provider-native-mock/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/chat_stream_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/responses_route.rs`

**Step 1: Add binary stream fixture behavior**

Make the native fixture stream deterministic bytes for:

- `audio.speech.create`
- `files.content`
- `videos.content`

**Step 2: Publish matching capabilities**

Add capability metadata for those operations so package and library manifests stay in sync.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-extension-host --test native_dynamic_runtime -q`

Expected: PASS

### Task 4: Add HTTP end-to-end native dynamic coverage and update docs

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/audio_speech_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/files_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/videos_route.rs`
- Modify: `README.md`
- Modify: `docs/api/compatibility-matrix.md`
- Modify: `docs/architecture/runtime-modes.md`

**Step 1: Add route coverage**

Prove native dynamic relay for:

- `/v1/audio/speech`
- `/v1/files/{file_id}/content`
- `/v1/videos/{video_id}/content`

**Step 2: Update docs**

Reflect that native dynamic runtime now supports:

- JSON provider operations
- chat and responses SSE
- binary stream passthrough for audio speech and file or video content

while lifecycle hooks and hot reload remain future work.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-interface-http --test audio_speech_route -q`
- `cargo test -p sdkwork-api-interface-http --test files_route -q`
- `cargo test -p sdkwork-api-interface-http --test videos_route -q`

Expected: PASS

### Task 5: Run verification and commit

**Files:**
- Modify all files above

**Step 1: Run verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `$env:CARGO_BUILD_JOBS='1'; cargo clippy --no-deps -p sdkwork-api-extension-host -p sdkwork-api-interface-http -p sdkwork-api-ext-provider-native-mock -p sdkwork-api-app-gateway --all-targets -- -D warnings`
- `$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0'; cargo test --workspace -q -j 1`

Expected: PASS

**Step 2: Commit**

```bash
git add README.md docs/api/compatibility-matrix.md docs/architecture/runtime-modes.md docs/plans/2026-03-14-native-dynamic-binary-streams-design.md docs/plans/2026-03-14-native-dynamic-binary-streams-implementation.md crates/sdkwork-api-ext-provider-native-mock crates/sdkwork-api-extension-host crates/sdkwork-api-interface-http crates/sdkwork-api-app-gateway
git commit -m "feat: add native dynamic binary stream support"
git push
```
