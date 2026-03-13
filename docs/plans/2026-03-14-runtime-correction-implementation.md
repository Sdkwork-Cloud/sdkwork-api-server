# Runtime Correction Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Close the main correctness gaps in the gateway runtime so data-plane tenancy, provider attribution, and extension runtime configuration all participate in real execution.

**Architecture:** Keep the current Axum + application-service + repository layering intact, but correct the runtime contract in three places. First, all stateful `/v1/*` handlers must derive tenant and project from the authenticated gateway API key. Second, usage attribution must distinguish real upstream relay from local emulation. Third, provider execution must consume persisted extension installation and instance state instead of rebuilding a pure built-in registry for every request.

**Tech Stack:** Rust, Axum, Tokio, serde_json, sqlx, existing SDKWork workspace crates

---

### Task 1: Add failing tests for remaining stateful tenancy propagation

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/gateway_auth_context.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/moderations_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/images_route.rs`

**Step 1: Write failing tests**

Add tests that prove:

- `/v1/moderations` requires a gateway API key in stateful mode
- `/v1/moderations` records usage and billing against the authenticated project instead of `project-1`
- `/v1/images/generations` also requires a gateway API key in stateful mode

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-interface-http --test gateway_auth_context -q`
- `cargo test -p sdkwork-api-interface-http --test moderations_route -q`
- `cargo test -p sdkwork-api-interface-http --test images_route -q`

Expected: FAIL because many stateful handlers still use hardcoded tenant and project literals.

### Task 2: Remove hardcoded data-plane tenancy from remaining stateful handlers

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`

**Step 1: Implement authenticated request context usage**

For every `_with_state_handler` that represents a data-plane call:

- require `AuthenticatedGatewayRequest`
- pass `request_context.tenant_id()` and `request_context.project_id()` to relay and local fallback paths
- remove usage of helper paths that still assume `project-1`

**Step 2: Update local helper responses**

Replace hardcoded tenant and project in local content and speech helpers with the authenticated request context or explicit parameters.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-interface-http --test gateway_auth_context -q`
- `cargo test -p sdkwork-api-interface-http --test moderations_route -q`
- `cargo test -p sdkwork-api-interface-http --test images_route -q`

Expected: PASS

### Task 3: Add failing tests for execution attribution and extension runtime load config

**Files:**
- Create: `crates/sdkwork-api-interface-http/tests/runtime_execution.rs`
- Modify: `crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs`

**Step 1: Write failing tests**

Add tests that prove:

- local fallback usage is recorded as a local runtime provider instead of `provider-openai-official`
- a provider can relay through a persisted extension instance whose `base_url` overrides the catalog provider record
- a disabled extension instance prevents upstream relay and falls back locally

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-interface-http --test runtime_execution -q`
- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch -q`

Expected: FAIL because provider attribution is still simulated after the fact and extension load plans are not consumed by the gateway runtime.

### Task 4: Route provider execution through persisted extension runtime state

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-usage/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-usage/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`

**Step 1: Introduce a runtime execution descriptor**

Add a small additive internal struct in the gateway application layer that carries:

- actual provider ID used for execution
- runtime key / extension ID
- resolved base URL
- resolved API key
- whether the execution is local fallback

**Step 2: Load extension runtime state**

When a provider is resolved:

- build an `ExtensionHost` from built-in manifests plus persisted installations and instances
- if a provider instance exists for `provider.id`, derive a load plan from it
- honor `enabled = false`
- let instance `base_url` override the catalog provider `base_url`
- keep backward-compatible behavior when no installation or instance exists

**Step 3: Fix usage attribution**

Use the actual execution descriptor for usage records:

- relay path records the concrete provider ID
- local fallback records a stable synthetic provider such as `sdkwork.local`
- stop re-simulating provider selection during usage persistence

**Step 4: Run focused tests**

Run:

- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch -q`
- `cargo test -p sdkwork-api-interface-http --test runtime_execution -q`
- `cargo test -p sdkwork-api-interface-http --test chat_route -q`
- `cargo test -p sdkwork-api-interface-http --test responses_route -q`
- `cargo test -p sdkwork-api-interface-http --test embeddings_route -q`

Expected: PASS

### Task 5: Externalize admin JWT signing secret and verify the corrected runtime

**Files:**
- Modify: `crates/sdkwork-api-config/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `services/admin-api-service/src/main.rs`
- Modify: `README.md`
- Modify: `docs/architecture/runtime-modes.md`

**Step 1: Add configuration support**

Add `admin_jwt_signing_secret` to runtime config and allow it to load from environment.

**Step 2: Wire admin API state**

Pass the configured signing secret into the admin router state instead of using a hardcoded development constant.

**Step 3: Run full verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace -q`

Expected: PASS

**Step 4: Commit**

```bash
git add docs/plans/2026-03-14-runtime-correction-implementation.md crates/sdkwork-api-interface-http crates/sdkwork-api-app-gateway crates/sdkwork-api-extension-host crates/sdkwork-api-app-routing crates/sdkwork-api-app-usage crates/sdkwork-api-domain-usage crates/sdkwork-api-config services/admin-api-service README.md docs/architecture/runtime-modes.md
git commit -m "feat: correct runtime execution and tenancy flow"
```
