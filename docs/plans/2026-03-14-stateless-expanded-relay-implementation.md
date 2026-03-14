# Stateless Expanded Relay Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Expand stateless upstream relay across the remaining OpenAI-compatible data-plane families so a configured stateless runtime behaves like a real relay gateway instead of a mostly emulated stub.

**Architecture:** Keep relay execution inside `sdkwork-api-interface-http` and reuse the existing `ProviderRequest` plus extension-host runtime resolution path. Add focused tests first for stateless relay behavior, then implement minimal handler changes and shared helper logic while preserving local fallback when no upstream runtime resolves.

**Tech Stack:** Rust, Axum, Reqwest, SQLx test harnesses, existing SDKWork provider and extension crates

---

### Task 1: Add failing stateless relay tests for remaining route families

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/files_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/uploads_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/audio_speech_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/images_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/moderations_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/realtime_sessions_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/assistants_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/threads_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/runs_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/conversations_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/vector_stores_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/vector_store_search_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/vector_store_files_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/vector_store_file_batches_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/batches_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/fine_tuning_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/webhooks_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/evals_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/videos_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/transcriptions_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/translations_route.rs`

**Step 1: Write the failing tests**

Add stateless tests that require a configured `StatelessGatewayConfig::with_upstream(...)` to relay for:

- files CRUD plus file content
- uploads create, part upload, complete, cancel
- audio speech binary output
- images generations, edits, variations
- moderations and realtime sessions
- assistants CRUD
- threads CRUD and messages
- thread runs, run steps, thread-and-run
- conversations and conversation items
- vector stores, search, files, and file batches
- batches
- fine-tuning jobs
- webhooks
- evals
- videos CRUD, content, remix
- audio transcriptions and translations

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-interface-http --test files_route -q`
- `cargo test -p sdkwork-api-interface-http --test uploads_route -q`
- `cargo test -p sdkwork-api-interface-http --test audio_speech_route -q`
- `cargo test -p sdkwork-api-interface-http --test images_route -q`
- `cargo test -p sdkwork-api-interface-http --test moderations_route -q`
- `cargo test -p sdkwork-api-interface-http --test realtime_sessions_route -q`
- `cargo test -p sdkwork-api-interface-http --test assistants_route -q`
- `cargo test -p sdkwork-api-interface-http --test threads_route -q`
- `cargo test -p sdkwork-api-interface-http --test runs_route -q`
- `cargo test -p sdkwork-api-interface-http --test conversations_route -q`
- `cargo test -p sdkwork-api-interface-http --test vector_stores_route -q`
- `cargo test -p sdkwork-api-interface-http --test vector_store_search_route -q`
- `cargo test -p sdkwork-api-interface-http --test vector_store_files_route -q`
- `cargo test -p sdkwork-api-interface-http --test vector_store_file_batches_route -q`
- `cargo test -p sdkwork-api-interface-http --test batches_route -q`
- `cargo test -p sdkwork-api-interface-http --test fine_tuning_route -q`
- `cargo test -p sdkwork-api-interface-http --test webhooks_route -q`
- `cargo test -p sdkwork-api-interface-http --test evals_route -q`
- `cargo test -p sdkwork-api-interface-http --test videos_route -q`
- `cargo test -p sdkwork-api-interface-http --test transcriptions_route -q`
- `cargo test -p sdkwork-api-interface-http --test translations_route -q`

Expected: FAIL because stateless handlers still return local fallback instead of configured upstream responses.

### Task 2: Implement stateless relay-first handler behavior

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`

**Step 1: Add small shared helper logic**

Introduce minimal helper routines that:

- execute stateless JSON relay and convert success to `Response`
- execute stateless stream relay and convert success to `Response`
- map relay errors to route-specific `502` responses

Keep these helpers narrow so they do not obscure route semantics.

**Step 2: Update stateless handlers family by family**

Change the stateless handlers to try upstream relay before local fallback for:

- files and uploads
- audio, images, moderations, realtime sessions
- assistants, threads, runs, conversations
- vector stores and nested resources
- batches, fine-tuning jobs, webhooks, evals, videos

For multipart routes, reuse the parsed request object for both relay and fallback.

For binary stream routes, use `relay_stateless_stream_request`.

**Step 3: Run focused tests**

Run the same targeted route tests from Task 1.

Expected: PASS

### Task 3: Update execution-truth documentation

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/api/compatibility-matrix.md`

**Step 1: Update docs**

Document:

- stateless mode now relays the broader OpenAI-compatible surface when an upstream runtime is configured
- local fallback remains available when no stateless upstream resolves
- the compatibility matrix labels for the affected families move from `emulated` to `relay`

**Step 2: Verify doc accuracy**

Re-read the affected route behavior and ensure documentation matches actual code paths.

### Task 4: Run full verification and commit

**Files:**
- Modify: repository worktree from previous tasks

**Step 1: Run verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`
- `pnpm --dir console build`
- `$env:CARGO_BUILD_JOBS='1'; cargo clippy --workspace --all-targets -- -D warnings`

Expected: all commands exit `0`

**Step 2: Commit**

```bash
git add docs/plans/2026-03-14-stateless-expanded-relay-design.md docs/plans/2026-03-14-stateless-expanded-relay-implementation.md crates/sdkwork-api-interface-http/src/lib.rs crates/sdkwork-api-interface-http/tests README.md README.zh-CN.md docs/architecture/runtime-modes.md docs/api/compatibility-matrix.md
git commit -m "feat: expand stateless relay coverage"
```
