# Payment Reconciliation Point Lookup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace reconciliation resolution's full-list scan with a primary-key point lookup across the payment storage layer.

**Architecture:** Extend `PaymentKernelStore` with a direct reconciliation line lookup method, implement it in sqlite and postgres using the existing row decoders, and switch admin resolution to use the new point read instead of scanning all reconciliation rows in memory.

**Tech Stack:** Rust, sqlx, SQLite, Postgres integration tests, Axum admin interface, cargo test

---

### Task 1: Lock point-lookup behavior with failing store tests

**Files:**
- Modify: `crates/sdkwork-api-storage-sqlite/tests/payment_store.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`

- [ ] **Step 1: Add a failing sqlite reconciliation point-lookup regression**
- [ ] **Step 2: Add a failing postgres reconciliation point-lookup regression**
- [ ] **Step 3: Assert unknown reconciliation ids return `None`**

- [ ] **Step 4: Run targeted store tests to verify failure**

Run: `cargo test --offline -p sdkwork-api-storage-sqlite --test payment_store -- --nocapture`
Expected: FAIL because the payment kernel store does not yet expose direct reconciliation lookup.

Run: `cargo test --offline -p sdkwork-api-storage-postgres --test integration_postgres -- --nocapture`
Expected: FAIL because the postgres store does not yet expose direct reconciliation lookup.

### Task 2: Implement storage-level reconciliation point reads

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`

- [ ] **Step 1: Add the new trait method to `PaymentKernelStore`**
- [ ] **Step 2: Implement sqlite point lookup by reconciliation line id**
- [ ] **Step 3: Implement postgres point lookup by reconciliation line id**
- [ ] **Step 4: Update admin reconciliation resolve to use the point lookup**

- [ ] **Step 5: Re-run targeted store and admin tests**

Run: `cargo test --offline -p sdkwork-api-storage-sqlite --test payment_store -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-storage-postgres --test integration_postgres -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: PASS

### Task 3: Re-verify the affected payment/admin slice

**Files:**
- Test: `crates/sdkwork-api-storage-sqlite/tests/payment_store.rs`
- Test: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`
- Test: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`

- [ ] **Step 1: Re-run sqlite payment store tests**

Run: `cargo test --offline -p sdkwork-api-storage-sqlite --test payment_store -- --nocapture`
Expected: PASS

- [ ] **Step 2: Re-run postgres integration tests**

Run: `cargo test --offline -p sdkwork-api-storage-postgres --test integration_postgres -- --nocapture`
Expected: PASS

- [ ] **Step 3: Re-run admin payment tests**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: PASS

- [ ] **Step 4: Re-run payment application regressions**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS
