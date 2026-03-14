# OpenAI Surface Parity Refresh Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Close the next confirmed OpenAI parity gaps by adding containers, completing remaining eval and fine-tuning subroutes, and correcting videos to the current official route surface while preserving legacy aliases.

**Architecture:** Extend the existing typed provider contract and route handlers. Add failing route tests first, then implement the minimal contract, provider, extension-host, gateway, and HTTP router changes required for stateful relay, stateless relay, local fallback, and stream passthrough.

**Tech Stack:** Rust, Axum, Reqwest, serde, SQLx, existing SDKWork provider and extension crates

---

### Task 1: Add failing HTTP tests for containers and remaining parity gaps

**Files:**
- Create: `crates/sdkwork-api-interface-http/tests/containers_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/evals_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/fine_tuning_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/videos_route.rs`

**Step 1: Write the failing tests**

Add tests that assert the gateway exposes:

- container create, list, retrieve, delete
- container file create, list, retrieve, delete, content
- eval run delete and cancel
- eval run output item list and retrieve
- fine-tuning job pause and resume
- fine-tuning checkpoint permission create, list or retrieve-compatible list, and delete
- official video routes for characters create and retrieve, edits create, and extensions create
- legacy video aliases still work

Cover:

- local compatible fallback behavior on the plain router
- stateless upstream relay behavior
- stateful relay behavior through configured provider data
- binary content passthrough behavior for container file content

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-interface-http --test containers_route -q`
- `cargo test -p sdkwork-api-interface-http --test evals_route -q`
- `cargo test -p sdkwork-api-interface-http --test fine_tuning_route -q`
- `cargo test -p sdkwork-api-interface-http --test videos_route -q`

Expected: FAIL because the new routes and provider request variants do not exist yet.

### Task 2: Add missing contract types and provider request variants

**Files:**
- Create: `crates/sdkwork-api-contract-openai/src/containers.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/lib.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/evals.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/fine_tuning.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/videos.rs`
- Modify: `crates/sdkwork-api-provider-core/src/lib.rs`

**Step 1: Add minimal additive request and response structs**

Model only the request and response shapes required by the handlers, provider dispatch, and local fallbacks.

**Step 2: Extend `ProviderRequest`**

Add variants for:

- containers
- container files and content
- eval run delete and cancel
- eval run output item list and retrieve
- fine-tuning pause and resume
- fine-tuning checkpoint permissions
- official video create-character, retrieve-character, edits, and extensions

**Step 3: Run compile-targeted tests**

Run:

- `cargo test -p sdkwork-api-provider-core -q`

Expected: PASS or compile forward to the next failing layer.

### Task 3: Implement provider and extension operation mapping

**Files:**
- Modify: `crates/sdkwork-api-provider-openai/src/lib.rs`
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`

**Step 1: Implement upstream OpenAI request mapping**

Map the new provider variants to the current official upstream paths:

- `/v1/containers...`
- `/v1/evals/.../runs/...`
- `/v1/fine_tuning/jobs/.../pause`
- `/v1/fine_tuning/jobs/.../resume`
- `/v1/fine_tuning/checkpoints/.../permissions...`
- `/v1/videos/characters`
- `/v1/videos/characters/{character_id}`
- `/v1/videos/edits`
- `/v1/videos/extensions`

**Step 2: Implement extension operation mapping**

Add stable operation names for the new variants and keep stream behavior intact for binary routes.

**Step 3: Run targeted tests**

Run:

- `cargo test -p sdkwork-api-provider-openai -q`
- `cargo test -p sdkwork-api-extension-host -q`

Expected: PASS

### Task 4: Implement gateway helpers, local fallbacks, and Axum routes

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`

**Step 1: Add relay helpers and local fallback functions**

Follow the existing naming and routing pattern so the new routes participate in stateful provider selection, stateless upstream execution, and local compatible fallback.

**Step 2: Wire official routes and legacy video aliases**

Add official video routes without removing the existing nested aliases in this batch.

**Step 3: Run focused route tests**

Run:

- `cargo test -p sdkwork-api-interface-http --test containers_route -q`
- `cargo test -p sdkwork-api-interface-http --test evals_route -q`
- `cargo test -p sdkwork-api-interface-http --test fine_tuning_route -q`
- `cargo test -p sdkwork-api-interface-http --test videos_route -q`

Expected: PASS

### Task 5: Update documentation and compatibility truth

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `docs/api/compatibility-matrix.md`
- Modify: `docs/architecture/runtime-modes.md`

**Step 1: Update documentation**

Document:

- container coverage
- completed eval and fine-tuning route coverage
- official video route surface plus legacy compatibility aliases
- new extension operation names

**Step 2: Run full verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`
- `pnpm --dir console build`
- `$env:CARGO_BUILD_JOBS='1'; cargo clippy --workspace --all-targets -- -D warnings`

Expected: all commands exit `0`

**Step 3: Commit**

```bash
git add docs/plans/2026-03-14-openai-surface-parity-refresh-design.md docs/plans/2026-03-14-openai-surface-parity-refresh-implementation.md crates/sdkwork-api-contract-openai/src/containers.rs crates/sdkwork-api-contract-openai/src/lib.rs crates/sdkwork-api-contract-openai/src/evals.rs crates/sdkwork-api-contract-openai/src/fine_tuning.rs crates/sdkwork-api-contract-openai/src/videos.rs crates/sdkwork-api-provider-core/src/lib.rs crates/sdkwork-api-provider-openai/src/lib.rs crates/sdkwork-api-extension-host/src/lib.rs crates/sdkwork-api-app-gateway/src/lib.rs crates/sdkwork-api-interface-http/src/lib.rs crates/sdkwork-api-interface-http/tests README.md README.zh-CN.md docs/api/compatibility-matrix.md docs/architecture/runtime-modes.md
git commit -m "feat: refresh openai surface parity"
```
