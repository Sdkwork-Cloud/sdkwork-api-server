# Payable Order Settlement Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove portal self-service settlement for payable orders while preserving trusted payment-callback fulfillment and idempotent order side effects.

**Architecture:** Split commerce settlement into portal-facing zero-pay settlement and trusted verified-payment settlement. Remove manual settlement from checkout surfaces, reject payable portal settlement attempts, and route payment callback fulfillment through the trusted path.

**Tech Stack:** Rust, axum, sqlx, cargo test

---

### Task 1: Lock the vulnerability with failing tests

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_commerce.rs`
- Modify: `crates/sdkwork-api-app-commerce/tests/commerce_checkout_bridge.rs`
- Modify: `crates/sdkwork-api-app-payment/tests/payment_order_service.rs`

- [ ] **Step 1: Write a failing portal regression proving payable checkout no longer exposes `settle_order`**
- [ ] **Step 2: Write a failing portal regression proving payable `POST /settle` is rejected and billing stays unchanged**
- [ ] **Step 3: Update checkout bridge regressions to expect no `manual_settlement` for payable orders**
- [ ] **Step 4: Run the focused tests to verify RED**

Run: `cargo test --offline -p sdkwork-api-interface-portal portal_commerce_pending_recharge_can_be_settled_or_canceled -- --nocapture`
Expected: FAIL because portal checkout still exposes manual settlement and payable `/settle` still succeeds.

### Task 2: Split settlement authority in commerce and payment apps

**Files:**
- Modify: `crates/sdkwork-api-app-commerce/src/lib.rs`
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`

- [ ] **Step 1: Add an internal trusted verified-payment settlement path**
- [ ] **Step 2: Restrict portal settlement to zero-payment orders**
- [ ] **Step 3: Remove `manual_settlement` from payable checkout methods in both checkout projections**

### Task 3: Rewire portal and callback tests

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_commerce.rs`
- Modify: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`

- [ ] **Step 1: Replace portal fake `settled` event fulfillment with verified callback-driven fulfillment in tests**
- [ ] **Step 2: Re-run focused tests to verify GREEN**

Run: `cargo test --offline -p sdkwork-api-interface-portal -- --nocapture`
Expected: PASS

### Task 4: Verify payment/admin regressions

**Files:**
- Test: `crates/sdkwork-api-app-payment`
- Test: `crates/sdkwork-api-interface-admin`

- [ ] **Step 1: Re-run payment regressions**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS

- [ ] **Step 2: Re-run admin payment regressions**

Run: `cargo test --offline -p sdkwork-api-interface-admin -- --nocapture`
Expected: PASS
