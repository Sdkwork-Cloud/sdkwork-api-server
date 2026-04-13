# Refund Request Reuse Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prevent duplicate creation of exact-match non-terminal refund requests while preserving valid distinct partial refunds.

**Architecture:** Add a reuse probe in `sdkwork-api-app-payment` before remaining-balance reservation, return the latest exact-match non-terminal refund order when present, and verify the behavior through payment-app and portal regressions.

**Tech Stack:** Rust, axum, sqlx, SQLite, cargo test

---

### Task 1: Reproduce duplicate refund-request creation

**Files:**
- Modify: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`

- [ ] **Step 1: Write the failing payment-app regression**

```rust
#[tokio::test]
async fn repeated_identical_partial_refund_request_reuses_pending_refund_order() {}
```

- [ ] **Step 2: Write the failing portal regression**

```rust
#[tokio::test]
async fn portal_repeated_refund_submission_reuses_pending_refund_order() {}
```

- [ ] **Step 3: Run targeted tests to verify they fail**

Run: `cargo test --offline -p sdkwork-api-app-payment repeated_identical_partial_refund_request_reuses_pending_refund_order -- --nocapture`
Expected: FAIL because a second identical request creates another refund order.

Run: `cargo test --offline -p sdkwork-api-interface-portal --test portal_order_center portal_repeated_refund_submission_reuses_pending_refund_order -- --nocapture`
Expected: FAIL because the second HTTP request creates another refund order.

### Task 2: Add reusable refund-request lookup

**Files:**
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`

- [ ] **Step 1: Add a helper to find the latest reusable non-terminal refund order**
- [ ] **Step 2: Reuse the matching refund order before remaining-balance reservation**
- [ ] **Step 3: Repair payment-order refund status to `pending` when reusing**

- [ ] **Step 4: Re-run targeted regressions**

Run: `cargo test --offline -p sdkwork-api-app-payment repeated_identical_partial_refund_request_reuses_pending_refund_order -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-interface-portal --test portal_order_center portal_repeated_refund_submission_reuses_pending_refund_order -- --nocapture`
Expected: PASS

### Task 3: Re-verify refund surfaces

**Files:**
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Test: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`
- Test: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`

- [ ] **Step 1: Run payment-app tests**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS

- [ ] **Step 2: Run portal order-center tests**

Run: `cargo test --offline -p sdkwork-api-interface-portal --test portal_order_center -- --nocapture`
Expected: PASS

- [ ] **Step 3: Run admin payment tests**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: PASS
