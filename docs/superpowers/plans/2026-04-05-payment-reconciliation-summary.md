# Payment Reconciliation Summary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose a monitoring-friendly admin summary for payment reconciliation anomalies so dashboards and operators can query compact lifecycle totals and active reason breakdowns.

**Architecture:** Add an authenticated admin summary endpoint, aggregate persisted reconciliation rows in the admin layer into lifecycle counters and active-reason buckets, and lock the contract with integration tests against SQLite-backed admin routes.

**Tech Stack:** Rust, Axum, serde serialization, SQLite-backed admin integration tests, cargo test

---

### Task 1: Lock reconciliation summary behavior with failing admin tests

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`

- [ ] **Step 1: Add a failing empty-state reconciliation summary regression**
- [ ] **Step 2: Add a failing populated reconciliation summary regression**
- [ ] **Step 3: Assert active-only reason breakdown semantics and timestamp aggregation**

- [ ] **Step 4: Run targeted admin payment tests to verify failure**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: FAIL because the admin router does not yet expose reconciliation summary aggregation.

### Task 2: Implement admin reconciliation summary aggregation

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`

- [ ] **Step 1: Add reconciliation summary response DTOs**
- [ ] **Step 2: Add the admin summary route and handler**
- [ ] **Step 3: Aggregate lifecycle totals from persisted reconciliation rows**
- [ ] **Step 4: Aggregate active-reason breakdown with stable sorting**

- [ ] **Step 5: Re-run targeted admin payment tests**

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
