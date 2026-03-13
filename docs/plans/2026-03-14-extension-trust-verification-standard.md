# Extension Trust Verification Standard Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a real extension trust model with manifest signature metadata, trusted signer policy, runtime load gating, and admin-facing trust observability for discovered extension packages.

**Architecture:** Keep discovery and trust evaluation separate so the control plane can always see external packages even when the runtime refuses to load them. Put trust metadata in `sdkwork-api-extension-core`, implement signature verification and trusted-signer policy in `sdkwork-api-extension-host`, and surface the resulting trust report through `sdkwork-api-app-extension`, `sdkwork-api-interface-admin`, and the runtime config environment model.

**Tech Stack:** Rust, serde, Axum, ed25519 signature verification, workspace tests

---

### Task 1: Add failing tests for trust metadata, signature verification, and runtime gating

**Files:**
- Modify: `crates/sdkwork-api-extension-core/tests/extension_standard.rs`
- Modify: `crates/sdkwork-api-extension-host/tests/discovery.rs`
- Modify: `crates/sdkwork-api-app-extension/tests/runtime_observability.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Modify: `crates/sdkwork-api-config/tests/config_loading.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- `ExtensionManifest` can carry trust metadata for publisher and detached signature material
- discovered packages produce a trust report with signature verification and load-allowance state
- the admin package listing returns trust details for discovered packages
- environment-driven config can parse trusted signer policy and signature enforcement toggles

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-extension-core --test extension_standard -q`
- `cargo test -p sdkwork-api-extension-host --test discovery -q`
- `cargo test -p sdkwork-api-app-extension --test runtime_observability -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes list_discovered_extension_packages_from_admin_api -- --exact`
- `cargo test -p sdkwork-api-config --test config_loading -q`

Expected: FAIL because manifests do not yet carry trust metadata, trust reports do not exist, admin DTOs do not expose trust state, and config does not parse trust policy.

### Task 2: Implement trust metadata and host verification policy

**Files:**
- Modify: `crates/sdkwork-api-extension-core/src/lib.rs`
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`
- Modify: `crates/sdkwork-api-extension-host/Cargo.toml`

**Step 1: Extend the manifest standard**

Add additive trust metadata:

- trust declaration with publisher identity
- signature block with algorithm, public key, and signature payload
- builder helpers for trust metadata

**Step 2: Add host trust verification**

Implement host-side trust policy and verification with:

- trusted signer allowlist
- runtime-specific signature requirements for connector and native-dynamic packages
- canonical package payload hashing and ed25519 verification
- load gating that refuses invalid or untrusted signed packages and optionally refuses unsigned packages by runtime

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-extension-core --test extension_standard -q`
- `cargo test -p sdkwork-api-extension-host --test discovery -q`
- `cargo test -p sdkwork-api-extension-host -q`

Expected: PASS

### Task 3: Surface trust state through app, admin, and runtime config layers

**Files:**
- Modify: `crates/sdkwork-api-app-extension/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-config/src/lib.rs`
- Modify: `crates/sdkwork-api-config/tests/config_loading.rs`

**Step 1: Expose trust report DTOs**

Add trust report data to discovered package records without creating a separate endpoint.

**Step 2: Enforce trust gating in runtime loading**

Update discovered package registration so external packages only enter the runtime host when the trust policy says they are loadable.

**Step 3: Parse runtime trust policy from environment**

Add config support for:

- trusted signer allowlist
- connector signature requirement
- native-dynamic signature requirement

**Step 4: Run focused tests**

Run:

- `cargo test -p sdkwork-api-app-extension --test runtime_observability -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes list_discovered_extension_packages_from_admin_api -- --exact`
- `cargo test -p sdkwork-api-config --test config_loading -q`
- `cargo test -p sdkwork-api-app-gateway -q`

Expected: PASS

### Task 4: Update docs and run full verification

**Files:**
- Modify: `README.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/api/compatibility-matrix.md`

**Step 1: Document the trust standard**

Document:

- manifest trust metadata
- trusted signer policy env vars
- admin trust observability
- runtime load behavior for unsigned or untrusted external packages

**Step 2: Run verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `$env:CARGO_BUILD_JOBS='1'; cargo clippy --no-deps -p sdkwork-api-extension-core -p sdkwork-api-extension-host -p sdkwork-api-app-extension -p sdkwork-api-interface-admin -p sdkwork-api-app-gateway -p sdkwork-api-config --all-targets -- -D warnings`
- `$env:CARGO_BUILD_JOBS='1'; cargo test --workspace -q -j 1`

Expected: PASS

**Step 3: Commit**

```bash
git add docs/plans/2026-03-14-extension-trust-verification-standard.md README.md docs/architecture/runtime-modes.md docs/api/compatibility-matrix.md crates/sdkwork-api-extension-core crates/sdkwork-api-extension-host crates/sdkwork-api-app-extension crates/sdkwork-api-interface-admin crates/sdkwork-api-app-gateway crates/sdkwork-api-config
git commit -m "feat: add extension trust verification policy"
git push
```
