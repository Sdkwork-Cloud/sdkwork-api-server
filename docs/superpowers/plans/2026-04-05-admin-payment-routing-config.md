# Admin Payment Routing Configuration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add admin API management for payment gateway accounts and payment channel policies so routing failover is operable in production.

**Architecture:** Extend the existing admin payment interface with list and upsert endpoints that read and write the canonical payment routing records already stored in sqlite and postgres. Keep the logic inside the admin interface crate for this slice, with focused validation and deterministic filtering/sorting helpers.

**Tech Stack:** Rust, axum, serde, sqlx-backed admin stores, cargo test

---

### Task 1: Lock admin routing management behavior with failing tests

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`

- [ ] **Step 1: Add a failing test for gateway account upsert and filtered list inspection**
- [ ] **Step 2: Add a failing test for channel policy upsert and filtered list inspection**
- [ ] **Step 3: Add a failing test for invalid routing payload rejection**
- [ ] **Step 4: Run the focused admin payment test target and verify failure**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: FAIL because the new admin payment routing endpoints and request handlers do not exist yet.

### Task 2: Implement admin payment routing endpoints and validation

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`

- [ ] **Step 1: Add request/query DTOs for gateway account and channel policy management**
- [ ] **Step 2: Register list and upsert routes under `/admin/payments/*`**
- [ ] **Step 3: Implement store-backed list helpers with deterministic filtering and sorting**
- [ ] **Step 4: Implement upsert handlers with validation and timestamp defaults**
- [ ] **Step 5: Re-run the focused admin payment test target**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: PASS

### Task 3: Re-verify the affected payment/admin slice

**Files:**
- Test: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Test: `crates/sdkwork-api-storage-sqlite/tests/payment_store.rs`

- [ ] **Step 1: Re-run admin payment tests**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: PASS

- [ ] **Step 2: Re-run payment callback processing tests**

Run: `cargo test --offline -p sdkwork-api-app-payment --test payment_callback_processing -- --nocapture`
Expected: PASS

- [ ] **Step 3: Re-run sqlite payment store tests**

Run: `cargo test --offline -p sdkwork-api-storage-sqlite --test payment_store -- --nocapture`
Expected: PASS

- [ ] **Step 4: Re-run the admin package and payment package for wider regression coverage**

Run: `cargo test --offline -p sdkwork-api-interface-admin -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS
