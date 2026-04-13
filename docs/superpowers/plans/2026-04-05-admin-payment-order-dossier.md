# Admin Payment Order Dossier Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an admin endpoint that returns the complete payment, refund, account, and finance evidence chain for one payment order.

**Architecture:** Build a dossier read model in `sdkwork-api-app-payment` over the existing commercial kernel traits, then expose it through `sdkwork-api-interface-admin` as `GET /admin/payments/orders/{payment_order_id}`. Keep the read path deterministic, id-filtered, and free of new write-side behavior.

**Tech Stack:** Rust, axum, serde, sqlx-backed sqlite/postgres admin stores, cargo test

---

### Task 1: Lock the dossier contract with failing admin tests

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`

- [ ] **Step 1: Add a failing test for a payment order dossier happy path**
- [ ] **Step 2: Add a failing test for refund/account/finance evidence in the dossier**
- [ ] **Step 3: Add a failing test for missing payment order returning 404**
- [ ] **Step 4: Run the focused admin payment test target and verify failure**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: FAIL because the dossier route and read model do not exist yet.

### Task 2: Implement the payment dossier read model

**Files:**
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`

- [ ] **Step 1: Add the dossier response struct in the payment app crate**
- [ ] **Step 2: Add deterministic helper functions for account-ledger ids produced by order grants and refund reversals**
- [ ] **Step 3: Implement the dossier loader using existing payment/account kernel methods**
- [ ] **Step 4: Filter and sort attempts, sessions, callbacks, transactions, refunds, reconciliation, ledger, and finance evidence**
- [ ] **Step 5: Return a not-found error when the payment order does not exist**

### Task 3: Expose the dossier from the admin interface

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`

- [ ] **Step 1: Register `GET /admin/payments/orders/{payment_order_id}`**
- [ ] **Step 2: Add the handler that calls the dossier loader on sqlite and postgres stores**
- [ ] **Step 3: Map missing-order errors to `404` and preserve the existing admin error pattern for everything else**
- [ ] **Step 4: Re-run the focused admin dossier tests**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: PASS

### Task 4: Re-verify the affected commercial payment slice

**Files:**
- Test: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Test: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`

- [ ] **Step 1: Format the touched Rust files**

Run: `cargo fmt --all`
Expected: PASS

- [ ] **Step 2: Re-run the focused admin payment test target**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: PASS

- [ ] **Step 3: Re-run the payment callback processing tests**

Run: `cargo test --offline -p sdkwork-api-app-payment --test payment_callback_processing -- --nocapture`
Expected: PASS

- [ ] **Step 4: Re-run the portal order center tests**

Run: `cargo test --offline -p sdkwork-api-interface-portal --test portal_order_center -- --nocapture`
Expected: PASS

- [ ] **Step 5: Re-run wider payment/admin package regressions**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-interface-admin -- --nocapture`
Expected: PASS
