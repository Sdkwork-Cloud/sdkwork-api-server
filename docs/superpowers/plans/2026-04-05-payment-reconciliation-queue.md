# Payment Reconciliation Queue Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn admin payment reconciliation listing into a usable operator queue with lifecycle filtering and unresolved-first ordering.

**Architecture:** Reuse persisted reconciliation lines as the single anomaly record, add an admin query model for `lifecycle=all|active|resolved`, and sort the in-memory admin list so unresolved lines are prioritized ahead of resolved history without changing storage contracts.

**Tech Stack:** Rust, Axum, serde query deserialization, SQLite-backed admin integration tests, cargo test

---

### Task 1: Lock queue behavior with failing admin tests

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`

- [ ] **Step 1: Add a failing regression for unresolved-first reconciliation ordering**
- [ ] **Step 2: Add failing regressions for `lifecycle=active` and `lifecycle=resolved`**
- [ ] **Step 3: Add a failing regression for invalid lifecycle values**

- [ ] **Step 4: Run the targeted admin payment test suite to verify failure**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: FAIL because the admin reconciliation endpoint does not yet support lifecycle filtering or queue prioritization.

### Task 2: Implement lifecycle-filtered admin reconciliation queue behavior

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`

- [ ] **Step 1: Add a reconciliation list query DTO for lifecycle filtering**
- [ ] **Step 2: Parse and validate lifecycle filter values**
- [ ] **Step 3: Filter reconciliation lines by lifecycle in the admin layer**
- [ ] **Step 4: Sort reconciliation lines with unresolved-first queue semantics**
- [ ] **Step 5: Return `400 Bad Request` for invalid lifecycle values**

- [ ] **Step 6: Re-run the targeted admin payment test suite**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: PASS

### Task 3: Re-verify the affected payment/admin slice

**Files:**
- Test: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`

- [ ] **Step 1: Re-run admin payment tests**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: PASS

- [ ] **Step 2: Re-run payment application regressions**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS
