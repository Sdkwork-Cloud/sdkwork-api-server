# OpenAPI Paste Retirement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** remove the remaining stale `utoipa-axum -> paste` lockfile advisory path while preserving the current admin and gateway OpenAPI contract behavior and eliminating stale portal OpenAPI runtime dependencies.

**Architecture:** finish the in-flight migration away from `utoipa-axum` instead of introducing a vendor. `sdkwork-api-interface-admin` and `sdkwork-api-interface-http` already moved to `#[derive(OpenApi)] + paths(...)`; the remaining work is to remove stale portal OpenAPI UI dependencies, regenerate `Cargo.lock`, and prove that audit plus route regressions still match the intended contract.

**Tech Stack:** Rust workspace manifests, Cargo lockfile regeneration, `cargo audit`, `cargo check`, Node-based lockfile regression tests, Axum + Utoipa OpenAPI route tests.

**Closure Update:** this plan is now complete. The active workspace graph no longer contains `paste` or `utoipa-axum`, and the current workspace `cargo audit --json --no-fetch --stale` result is clean. Historical step wording below is retained as execution context, not current unresolved risk.

---

## File Map

- Modify: `Cargo.toml`
  - keep workspace dependency declarations aligned with the already-removed `utoipa-axum` dependency.
- Modify: `Cargo.lock`
  - record the new dependency graph after `paste` retirement and portal dependency cleanup.
- Modify: `scripts/check-rust-dependency-audit.policy.json`
  - stop allowing `RUSTSEC-2024-0436`.
- Modify: `scripts/check-rust-dependency-audit.test.mjs`
  - reject `paste` in `Cargo.lock`.
- Modify: `crates/sdkwork-api-interface-portal/Cargo.toml`
  - remove stale `utoipa-swagger-ui` dependency.
- Test: `crates/sdkwork-api-interface-admin/tests/openapi_route.rs`
  - narrow regression proof that admin OpenAPI generation still works.
- Test: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`
  - narrow regression proof that gateway OpenAPI generation still works.
- Modify: `docs/superpowers/specs/2026-04-15-openapi-paste-retirement-design.md`
  - design source of truth for this hardening slice.

### Task 1: Tighten the regression gates first

**Files:**
- Modify: `scripts/check-rust-dependency-audit.policy.json`
- Modify: `scripts/check-rust-dependency-audit.test.mjs`

- [ ] **Step 1: Remove the `paste` advisory allowlist**

Delete `RUSTSEC-2024-0436` from the audit policy so the repository now treats the advisory as unresolved debt.

- [ ] **Step 2: Add an explicit lockfile regression**

Extend the lockfile test so it fails if `Cargo.lock` still contains:

```text
name = "paste"
```

- [ ] **Step 3: Run the narrow audit regression**

Run:

```bash
node --test scripts/check-rust-dependency-audit.test.mjs
```

Expected: FAIL before implementation because the policy still sees `paste` in `Cargo.lock`.

### Task 2: Remove stale portal OpenAPI generator dependencies

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/Cargo.toml`

- [ ] **Step 1: Drop dead dependencies**

Remove `utoipa-swagger-ui.workspace = true` from the portal interface crate because the portal publishes a static OpenAPI document and does not use that crate in source.

- [ ] **Step 2: Verify the crate manifest still resolves**

Run:

```bash
cargo check -p sdkwork-api-interface-portal
```

Expected: PASS or fail only on the remaining `paste` path from other crates, not because of portal manifest breakage.

### Task 3: Regenerate the lockfile from the real dependency graph

**Files:**
- Modify: `Cargo.lock`

- [ ] **Step 1: Regenerate `Cargo.lock`**

Run a Cargo-managed lockfile rewrite so stale packages not present in the active graph are pruned.

- [ ] **Step 2: Verify `paste` and `utoipa-axum` disappear from the lockfile**

Run:

```bash
cargo tree -i utoipa-axum --workspace
cargo tree -i paste --workspace
```

Expected: both report that the package ID does not match any package in the workspace.

- [ ] **Step 3: Verify the narrow admin and gateway OpenAPI regressions**

Run:

```bash
cargo test -p sdkwork-api-interface-admin --test openapi_route -q
cargo test -p sdkwork-api-interface-http --test openapi_route -q
```

Expected: both PASS with no OpenAPI route inventory drift.

### Task 4: Verify the dependency graph actually changed

**Files:**
- Modify: `Cargo.lock`

- [ ] **Step 1: Confirm `paste` is gone from the graph**

Run:

```bash
cargo tree -i paste --workspace
```

Expected: Cargo reports no package named `paste` in the workspace dependency graph.

- [ ] **Step 2: Confirm `utoipa-axum` now resolves from the vendor**

Run:

```bash
cargo tree -i utoipa-axum --workspace
```

Expected: Cargo reports that `utoipa-axum` is not part of the current workspace graph.

### Task 5: Fresh final verification

**Files:**
- Modify: none unless verification exposes a real defect

- [ ] **Step 1: Run the dependency audit suite**

Run:

```bash
node --test scripts/check-rust-dependency-audit.test.mjs scripts/check-rust-verification-matrix.test.mjs scripts/rust-verification-workflow.test.mjs
node scripts/check-rust-verification-matrix.mjs --group dependency-audit
cargo audit --json --no-fetch --stale
```

Expected:

- audit tests pass
- dependency-audit matrix group passes
- `cargo audit` no longer reports `RUSTSEC-2024-0436`

- [ ] **Step 2: Run affected compile verification**

Run:

```bash
cargo check -p sdkwork-api-interface-admin
cargo check -p sdkwork-api-interface-http
cargo check -p sdkwork-api-interface-portal
```

Expected: PASS for all three interface crates.

- [ ] **Step 3: Record closure status honestly**

Record that this slice closed the `paste` advisory path and that the active workspace audit graph is now clean. Any earlier references to `RUSTSEC-2026-0097` belonged to pre-closure workspace state, not the final result of this plan.
