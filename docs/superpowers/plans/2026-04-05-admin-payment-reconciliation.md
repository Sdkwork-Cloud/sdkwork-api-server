# Admin Payment Reconciliation Visibility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose persisted payment reconciliation evidence through a read-only admin API so operators can inspect refund conflict anomalies without database access.

**Architecture:** Extend the payment store trait with an all-record reconciliation query, implement it for SQLite and Postgres, and add a new admin route returning `ReconciliationMatchSummaryRecord` values sorted by newest first.

**Tech Stack:** Rust, axum, sqlx, SQLite, Postgres, cargo test

---

### Task 1: Reproduce the missing admin visibility

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`

- [ ] **Step 1: Write the failing admin route test**

```rust
#[tokio::test]
async fn admin_payment_routes_list_reconciliation_lines() {}
```

- [ ] **Step 2: Run the targeted test to verify it fails**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments admin_payment_routes_list_reconciliation_lines -- --nocapture`
Expected: FAIL because the route does not exist yet.

### Task 2: Add store and admin route support

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`

- [ ] **Step 1: Add a store method for listing all reconciliation lines**
- [ ] **Step 2: Implement the method in SQLite**
- [ ] **Step 3: Implement the method in Postgres**
- [ ] **Step 4: Add the admin route and loader helper**

- [ ] **Step 5: Re-run the targeted admin test**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments admin_payment_routes_list_reconciliation_lines -- --nocapture`
Expected: PASS

### Task 3: Re-verify payment operator surfaces

**Files:**
- Test: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`

- [ ] **Step 1: Run admin payment tests**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: PASS

- [ ] **Step 2: Run payment app tests**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS
