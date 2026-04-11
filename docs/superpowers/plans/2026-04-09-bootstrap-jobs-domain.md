# Bootstrap Jobs Domain Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the repository bootstrap system with an idempotent `jobs` data domain so dev and prod installs start with realistic async job data and dashboards can render immediately.

**Architecture:** Reuse the existing bootstrap plugin pattern: add a `jobs` manifest section, a typed jobs bundle loader, validation for cross-record consistency, and four ordered registry stages that upsert job records into storage. Seed `/data/jobs` with stable IDs and environment-specific demo workflows aligned to existing providers, models, routing, commerce, and billing data.

**Tech Stack:** Rust, Tokio tests, SQLite-backed `AdminStore`, repository `/data` JSON manifests.

---

### Task 1: Add failing runtime bootstrap tests

**Files:**
- Modify: `crates/sdkwork-api-app-runtime/src/tests.rs`
- Modify: `crates/sdkwork-api-product-runtime/tests/product_runtime.rs`

- [ ] **Step 1: Write the failing test**

Add assertions for:
- bootstrap import of async jobs, attempts, assets, callbacks
- repeated startup remains idempotent
- invalid jobs bundle with a missing job reference is rejected
- repository `prod` and `dev` profiles expose seeded jobs data

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
Expected: FAIL because the bootstrap system does not yet load or validate `jobs`.

- [ ] **Step 3: Write minimal implementation**

Add manifest support, bundle loading, validation, registry stages, and repository data needed to satisfy the tests.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/plans/2026-04-09-bootstrap-jobs-domain.md crates/sdkwork-api-app-runtime/src/tests.rs crates/sdkwork-api-product-runtime/tests/product_runtime.rs
git commit -m "test: cover bootstrap jobs domain"
```

### Task 2: Wire jobs into the bootstrap framework

**Files:**
- Modify: `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`
- Modify: `crates/sdkwork-api-app-runtime/src/bootstrap_data/registry.rs`

- [ ] **Step 1: Write the failing test**

Covered by Task 1.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
Expected: FAIL on missing `jobs` support.

- [ ] **Step 3: Write minimal implementation**

Add:
- `jobs` profile manifest field
- typed jobs bundle
- `BootstrapDataPack` job collections
- validation for unique IDs, attempt/asset/callback parent references, JSON validity, and optional provider/model references
- ordered registry stages for jobs, attempts, assets, callbacks

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs crates/sdkwork-api-app-runtime/src/bootstrap_data/registry.rs
git commit -m "feat: add bootstrap jobs domain"
```

### Task 3: Seed repository jobs data

**Files:**
- Create: `data/jobs/default.json`
- Create: `data/jobs/dev.json`
- Modify: `data/profiles/prod.json`
- Modify: `data/profiles/dev.json`

- [ ] **Step 1: Write the failing test**

Covered by Task 1 repository bootstrap assertions.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`
Expected: FAIL because repository profiles do not yet include jobs data.

- [ ] **Step 3: Write minimal implementation**

Seed globally useful production examples and richer dev-only sandbox examples with stable IDs and realistic provider/model combinations.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add data/jobs/default.json data/jobs/dev.json data/profiles/prod.json data/profiles/dev.json
git commit -m "feat: seed bootstrap jobs data"
```

### Task 4: Verify regression safety

**Files:**
- Modify if needed: `crates/sdkwork-api-app-runtime/src/tests.rs`
- Modify if needed: `crates/sdkwork-api-product-runtime/tests/product_runtime.rs`

- [ ] **Step 1: Run focused verification**

Run: `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
Expected: PASS

- [ ] **Step 2: Run runtime verification**

Run: `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`
Expected: PASS

- [ ] **Step 3: Run broader regression verification**

Run:
- `cargo test -p sdkwork-api-app-runtime --lib`
- `cargo test -p sdkwork-api-product-runtime`

Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add crates/sdkwork-api-app-runtime/src/tests.rs crates/sdkwork-api-product-runtime/tests/product_runtime.rs
git commit -m "test: verify bootstrap jobs integration"
```
