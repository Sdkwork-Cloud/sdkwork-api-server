# Pending Order Reuse Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reuse an existing matching payable `pending_payment` order instead of creating duplicate orders for repeated identical checkout requests.

**Architecture:** Add a quote-aware pending-order lookup inside `submit_portal_commerce_order(...)`, limited to payable orders and scoped by project, user, quote economics, and applied coupon. Keep portal checkout synchronization unchanged so reused orders still repair missing canonical payment artifacts automatically.

**Tech Stack:** Rust, axum, sqlx, cargo test

---

### Task 1: Lock duplicate-order behavior with failing tests

**Files:**
- Modify: `crates/sdkwork-api-app-commerce/tests/commerce_order_settlement.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_commerce.rs`

- [ ] **Step 1: Write a failing commerce regression proving repeated identical payable submissions reuse the same pending order**
- [ ] **Step 2: Write a failing portal regression proving repeated identical payable create requests return the same order id and keep a single order-center entry**
- [ ] **Step 3: Run the focused tests to verify RED**

Run: `cargo test --offline -p sdkwork-api-app-commerce repeated_identical_payable_submission_reuses_pending_order -- --nocapture`
Expected: FAIL because a second submission still creates a different order id.

Run: `cargo test --offline -p sdkwork-api-interface-portal portal_commerce_reuses_existing_pending_payable_order_for_repeat_create -- --nocapture`
Expected: FAIL because repeated portal create calls still create duplicate pending orders.

### Task 2: Implement quote-aware pending-order reuse

**Files:**
- Modify: `crates/sdkwork-api-app-commerce/src/lib.rs`

- [ ] **Step 1: Add a helper that finds the newest matching payable `pending_payment` order for the same quote intent**
- [ ] **Step 2: Call the helper before inserting a new payable order**
- [ ] **Step 3: Keep zero-pay fulfillment flows on their existing path**

### Task 3: Verify portal and payment surfaces stay singular

**Files:**
- Test: `crates/sdkwork-api-app-commerce`
- Test: `crates/sdkwork-api-interface-portal`
- Test: `crates/sdkwork-api-app-payment`

- [ ] **Step 1: Re-run focused tests to verify GREEN**

Run: `cargo test --offline -p sdkwork-api-app-commerce --test commerce_order_settlement -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-interface-portal --test portal_commerce -- --nocapture`
Expected: PASS

- [ ] **Step 2: Re-run broader regressions**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-interface-admin -- --nocapture`
Expected: PASS
