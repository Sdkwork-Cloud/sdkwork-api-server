# Admin Auth, Catalog Binding, and Extension Instance Iteration Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Finalize signed admin JWT protection, then evolve the catalog and extension runtime so providers can bind to multiple channels and extensions can be installed as configurable instances.

**Architecture:** Treat the current admin JWT work as a stabilization task, not a redesign. After that, extend the catalog model additively with provider-channel bindings and richer model metadata, then extend the extension host with installation and instance mounting so the gateway can move from built-in adapters toward a pluggable runtime.

**Tech Stack:** Rust, Axum, Tokio, sqlx, serde, jsonwebtoken, SQLite, PostgreSQL

---

### Task 1: Finalize admin JWT auth and route protection

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/sdkwork-api-app-identity/Cargo.toml`
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-app-identity/tests/jwt_and_api_key.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Create: `crates/sdkwork-api-interface-admin/tests/admin_auth_guard.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/auth_and_project_routes.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

**Step 1: Run the auth-focused tests**

Run:

- `cargo test -p sdkwork-api-app-identity -q`
- `cargo test -p sdkwork-api-interface-admin --test admin_auth_guard -q`

Expected: PASS for core auth behavior.

**Step 2: Verify protected handlers require bearer JWT**

Run:

- `cargo test -p sdkwork-api-interface-admin --test auth_and_project_routes -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`

Expected: PASS after every protected request includes a valid login token.

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates/sdkwork-api-app-identity crates/sdkwork-api-interface-admin
git commit -m "feat: harden admin jwt auth"
```

### Task 2: Add catalog-level provider-channel bindings and richer model metadata

**Files:**
- Modify: `crates/sdkwork-api-domain-catalog/src/lib.rs`
- Modify: `crates/sdkwork-api-app-catalog/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Create: `crates/sdkwork-api-domain-catalog/tests/provider_channel_binding.rs`
- Create: `crates/sdkwork-api-storage-sqlite/tests/catalog_bindings.rs`

**Step 1: Write a failing domain test for multi-channel bindings**

Run:

- `cargo test -p sdkwork-api-domain-catalog --test provider_channel_binding -q`

Expected: FAIL because the binding and richer model concepts do not exist yet.

**Step 2: Add minimal additive domain and storage support**

Implement:

- `ProviderChannelBinding`
- richer `ModelVariant` capability metadata
- additive storage APIs and schema for bindings

**Step 3: Verify focused catalog tests**

Run:

- `cargo test -p sdkwork-api-domain-catalog --test provider_channel_binding -q`
- `cargo test -p sdkwork-api-storage-sqlite --test catalog_bindings -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`

Expected: PASS

**Step 4: Commit**

```bash
git add crates/sdkwork-api-domain-catalog crates/sdkwork-api-app-catalog crates/sdkwork-api-storage-core crates/sdkwork-api-storage-sqlite crates/sdkwork-api-storage-postgres crates/sdkwork-api-interface-admin
git commit -m "feat: enrich catalog with channel bindings and model metadata"
```

### Task 3: Add extension installation and instance mounting

**Files:**
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`
- Create: `crates/sdkwork-api-extension-host/tests/instance_mounting.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`

**Step 1: Write a failing host test for instance mounting**

Run:

- `cargo test -p sdkwork-api-extension-host --test instance_mounting -q`

Expected: FAIL because the host has no installation or instance model.

**Step 2: Add minimal installation and instance state**

Implement:

- extension package or installation descriptors
- instance mounting per extension ID
- configuration payload and enablement state

**Step 3: Verify**

Run:

- `cargo test -p sdkwork-api-extension-host --test instance_mounting -q`
- `cargo test -p sdkwork-api-interface-admin --test admin_auth_guard -q`

Expected: PASS

**Step 4: Commit**

```bash
git add crates/sdkwork-api-extension-host crates/sdkwork-api-storage-core crates/sdkwork-api-storage-sqlite crates/sdkwork-api-storage-postgres crates/sdkwork-api-interface-admin
git commit -m "feat: add extension installation and instance config"
```

### Task 4: Verification checkpoint

**Files:**
- Modify: `README.md`
- Modify: `docs/api/compatibility-matrix.md`

**Step 1: Run verification**

Run:

- `cargo fmt --all`
- `cargo test -p sdkwork-api-interface-admin -q`
- `cargo test -p sdkwork-api-extension-host -q`
- `cargo test -p sdkwork-api-interface-http -q`

Expected: PASS

**Step 2: Update runtime status docs and commit**

```bash
git add README.md docs/api/compatibility-matrix.md
git commit -m "docs: update extension runtime status"
```
