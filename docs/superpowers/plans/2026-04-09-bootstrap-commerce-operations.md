# Bootstrap Commerce Operations Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deepen repository bootstrap data so dev and prod startup seed complete commerce operations records beyond orders and payment events, including payment attempts, webhook inbox state, refunds, and reconciliation artifacts.

**Architecture:** Extend the existing `commerce` bootstrap bundle rather than opening a new domain. Keep related operational records together in `/data/commerce`, add validation for cross-record integrity and JSON payloads, and apply records through ordered registry stages that preserve idempotent startup behavior.

**Tech Stack:** Rust, Tokio tests, SQLite-backed `AdminStore`, repository `/data` JSON manifests.

---

### Task 1: Add failing tests

**Files:**
- Modify: `crates/sdkwork-api-app-runtime/src/tests.rs`
- Modify: `crates/sdkwork-api-product-runtime/tests/product_runtime.rs`

- [ ] **Step 1: Write the failing test**

Cover:
- bootstrap import of payment attempts, webhook inbox records, refunds, reconciliation runs, reconciliation items
- repeated startup remains idempotent for those records
- invalid refund or payment attempt references are rejected
- repository `prod` and `dev` profiles expose seeded operations data

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
Expected: FAIL because bootstrap does not yet load extended commerce operations data.

- [ ] **Step 3: Write minimal implementation**

Add only the code and data needed to satisfy the tests.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/plans/2026-04-09-bootstrap-commerce-operations.md crates/sdkwork-api-app-runtime/src/tests.rs crates/sdkwork-api-product-runtime/tests/product_runtime.rs
git commit -m "test: cover bootstrap commerce operations"
```

### Task 2: Extend bootstrap commerce bundle and validation

**Files:**
- Modify: `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`
- Modify: `crates/sdkwork-api-app-runtime/src/bootstrap_data/registry.rs`

- [ ] **Step 1: Re-run failing tests**

Run: `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
Expected: FAIL on missing commerce operations support.

- [ ] **Step 2: Write minimal implementation**

Add:
- commerce bundle fields for payment attempts, webhook inbox records, refunds, reconciliation runs, reconciliation items
- data pack collections
- validation for unique IDs, order/payment method references, idempotency keys, payload JSON validity, and time ordering
- ordered registry stages for the new records

- [ ] **Step 3: Run tests**

Run: `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs crates/sdkwork-api-app-runtime/src/bootstrap_data/registry.rs
git commit -m "feat: bootstrap commerce operations"
```

### Task 3: Seed repository commerce operations data

**Files:**
- Modify: `data/commerce/default.json`
- Modify: `data/commerce/dev.json`

- [ ] **Step 1: Re-run repository tests**

Run: `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`
Expected: FAIL because repository commerce files do not yet include operations data.

- [ ] **Step 2: Write minimal implementation**

Seed:
- prod examples for hosted checkout, webhook receipt, refund, and reconciliation findings
- dev examples for sandbox/manual-review flows and pending operations

- [ ] **Step 3: Run tests**

Run: `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add data/commerce/default.json data/commerce/dev.json
git commit -m "feat: seed commerce operations bootstrap data"
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
git add crates/sdkwork-api-app-runtime/src/tests.rs crates/sdkwork-api-product-runtime/tests/product_runtime.rs data/commerce/default.json data/commerce/dev.json
git commit -m "test: verify commerce operations bootstrap"
```
