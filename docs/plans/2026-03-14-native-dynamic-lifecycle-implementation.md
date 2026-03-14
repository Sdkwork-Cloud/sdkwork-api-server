# Native Dynamic Lifecycle Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add optional lifecycle hooks, health contracts, and runtime status observability for `native_dynamic` extensions without breaking the existing ABI.

**Architecture:** Extend the native dynamic ABI with optional lifecycle exports, track loaded runtimes in the host through a lightweight registry, and expose one normalized runtime status view across connector and native dynamic modes through the app and admin layers.

**Tech Stack:** Rust, Axum, tokio, serde_json, libloading

---

### Task 1: Add failing lifecycle and status tests

**Files:**
- Modify: `crates/sdkwork-api-extension-host/tests/native_dynamic_runtime.rs`
- Modify: `crates/sdkwork-api-app-extension/tests/runtime_observability.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- native dynamic runtime calls `init` when loaded
- native dynamic runtime exposes health through a host status API
- native dynamic runtime calls `shutdown` during cleanup
- admin runtime status output is runtime-neutral and includes native dynamic entries

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-extension-host --test native_dynamic_runtime -q`
- `cargo test -p sdkwork-api-app-extension --test runtime_observability -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`

Expected: FAIL because the ABI has no lifecycle hooks yet and runtime statuses are still connector-only.

### Task 2: Extend the ABI with optional lifecycle payloads and results

**Files:**
- Modify: `crates/sdkwork-api-extension-abi/src/lib.rs`

**Step 1: Add lifecycle ABI symbols and shared JSON contracts**

Introduce:

- optional symbol constants for `init`, `health_check`, and `shutdown`
- lifecycle payload type for package-runtime context
- health result type with `healthy`, optional `message`, and optional `details`
- generic lifecycle result type for init and shutdown

**Step 2: Run focused tests**

Run:

- `cargo test -p sdkwork-api-extension-abi -q`

Expected: PASS if the new types serialize cleanly and no existing ABI behavior regresses.

### Task 3: Implement host-side lifecycle loading and runtime registry

**Files:**
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`
- Modify: `crates/sdkwork-api-extension-host/tests/native_dynamic_runtime.rs`

**Step 1: Load optional lifecycle symbols**

When loading a native dynamic library:

- keep required symbol resolution unchanged
- detect optional lifecycle exports
- call `init` once with package-runtime context
- fail load if `init` returns an error result

**Step 2: Track runtime status**

Add host-managed native runtime status tracking that records:

- runtime kind
- extension identity
- display name
- library path
- running and healthy state
- lifecycle support flags
- last message or error

**Step 3: Add cleanup hooks**

Support explicit cleanup and drop-time shutdown so tests and future host shutdown flows can release runtimes deterministically.

**Step 4: Run focused tests**

Run:

- `cargo test -p sdkwork-api-extension-host --test native_dynamic_runtime -q`

Expected: PASS

### Task 4: Normalize runtime status records across connector and native dynamic

**Files:**
- Modify: `crates/sdkwork-api-app-extension/src/lib.rs`
- Modify: `crates/sdkwork-api-app-extension/tests/runtime_observability.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

**Step 1: Replace connector-only status records**

Expose one runtime-neutral status record that can represent:

- connector process runtimes
- native dynamic in-process runtimes

**Step 2: Keep backward-compatible semantics where practical**

Connector records should still include:

- `base_url`
- `health_url`
- `process_id`

Native dynamic records should include:

- `library_path`
- `supports_health_check`
- `supports_shutdown`

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-app-extension --test runtime_observability -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`

Expected: PASS

### Task 5: Extend the native mock plugin to exercise lifecycle hooks

**Files:**
- Modify: `crates/sdkwork-api-ext-provider-native-mock/src/lib.rs`

**Step 1: Add optional lifecycle exports**

Implement deterministic fixture behavior for:

- init success marker
- health check success marker
- shutdown success marker

**Step 2: Keep execution behavior unchanged**

Do not regress existing JSON, SSE, or binary stream fixture operations.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-ext-provider-native-mock -q`
- `cargo test -p sdkwork-api-extension-host --test native_dynamic_runtime -q`

Expected: PASS

### Task 6: Update docs and run full verification

**Files:**
- Modify: `README.md`
- Modify: `docs/api/compatibility-matrix.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/plans/2026-03-14-native-dynamic-lifecycle-design.md`
- Modify: `docs/plans/2026-03-14-native-dynamic-lifecycle-implementation.md`

**Step 1: Update docs**

Reflect that `native_dynamic` now supports:

- JSON and stream execution
- optional lifecycle hooks
- runtime health contracts
- runtime status visibility through the admin plane

while hot reload and health-informed routing remain future work.

**Step 2: Run verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `$env:CARGO_BUILD_JOBS='1'; cargo clippy --no-deps -p sdkwork-api-extension-abi -p sdkwork-api-extension-host -p sdkwork-api-ext-provider-native-mock -p sdkwork-api-app-extension -p sdkwork-api-interface-admin --all-targets -- -D warnings`
- `$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0'; cargo test --workspace -q -j 1`

Expected: PASS

**Step 3: Commit**

```bash
git add README.md docs/api/compatibility-matrix.md docs/architecture/runtime-modes.md docs/plans/2026-03-14-native-dynamic-lifecycle-design.md docs/plans/2026-03-14-native-dynamic-lifecycle-implementation.md crates/sdkwork-api-extension-abi crates/sdkwork-api-extension-host crates/sdkwork-api-ext-provider-native-mock crates/sdkwork-api-app-extension crates/sdkwork-api-interface-admin
git commit -m "feat: add native dynamic lifecycle hooks"
git push
```
