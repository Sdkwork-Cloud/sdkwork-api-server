# Portal Canonical Payment Checkout Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Automatically prepare canonical payment records for every paid portal commerce order and switch portal payment/account surfaces onto a stable portal-to-canonical subject bridge.

**Architecture:** Keep the existing portal commerce APIs additive and unchanged, but insert a payment-side identity bridge plus idempotent checkout preparation helper. Trigger that helper from the portal interface on order creation and checkout-session load, and tighten read-model refundability semantics.

**Tech Stack:** Rust, axum, sqlx, cargo test

---

### Task 1: Lock the new checkout behavior with failing tests

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`

- [ ] **Step 1: Add a failing regression for paid portal orders auto-creating canonical payment artifacts**
- [ ] **Step 2: Assert pending paid orders are not shown as refundable before capture**
- [ ] **Step 3: Run the focused portal regression to verify RED**

Run: `cargo test --offline -p sdkwork-api-interface-portal portal_paid_order_prepares_canonical_payment_artifacts -- --nocapture`
Expected: FAIL because canonical payment artifacts are not created automatically yet.

### Task 2: Add portal identity bridge and payment preparation helpers

**Files:**
- Modify: `crates/sdkwork-api-app-payment/Cargo.toml`
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`

- [ ] **Step 1: Add a stable portal-to-canonical subject helper**
- [ ] **Step 2: Add an idempotent portal checkout preparation helper**
- [ ] **Step 3: Change account-history loading to use the new identity bridge**
- [ ] **Step 4: Tighten refundable amount calculation for non-refundable payment statuses**

### Task 3: Wire portal handlers to prepare canonical checkout

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`

- [ ] **Step 1: Prepare canonical payment artifacts after paid order creation**
- [ ] **Step 2: Prepare canonical payment artifacts during checkout-session reads as a backfill path**
- [ ] **Step 3: Pass portal user identity into account-history loading**

### Task 4: Verify the regression surface

**Files:**
- Test: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`
- Test: `crates/sdkwork-api-interface-portal/tests/portal_commerce.rs`
- Test: `crates/sdkwork-api-app-payment`

- [ ] **Step 1: Run the focused portal regression**

Run: `cargo test --offline -p sdkwork-api-interface-portal portal_paid_order_prepares_canonical_payment_artifacts -- --nocapture`
Expected: PASS

- [ ] **Step 2: Run the full portal interface suite**

Run: `cargo test --offline -p sdkwork-api-interface-portal -- --nocapture`
Expected: PASS

- [ ] **Step 3: Re-run payment app regressions**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS
