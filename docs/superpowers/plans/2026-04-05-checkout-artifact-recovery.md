# Checkout Artifact Recovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make portal order-center reads repair missing canonical payment artifacts without downgrading verified payment progress.

**Architecture:** Harden `ensure_commerce_payment_checkout(...)` into a recovery-safe reconciler that preserves advanced persisted payment state and recreates only missing or forward-moving artifacts. Then invoke that reconciler from the portal order-center read path so partially prepared checkouts can self-heal on demand.

**Tech Stack:** Rust, axum, sqlx, cargo test

---

### Task 1: Lock recovery requirements with failing tests

**Files:**
- Modify: `crates/sdkwork-api-app-payment/tests/payment_order_service.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`

- [ ] **Step 1: Write a failing payment-service regression proving advanced captured state is preserved while missing attempt/session artifacts are rebuilt**
- [ ] **Step 2: Write a failing portal regression proving `GET /portal/commerce/order-center` repairs missing canonical payment artifacts for a payable order**
- [ ] **Step 3: Run the focused tests to verify RED**

Run: `cargo test --offline -p sdkwork-api-app-payment preserves_advanced_payment_state_while_repairing_missing_checkout_artifacts -- --nocapture`
Expected: FAIL because checkout ensure logic still overwrites existing state instead of reconciling it.

Run: `cargo test --offline -p sdkwork-api-interface-portal portal_order_center_repairs_missing_checkout_artifacts_for_payable_orders -- --nocapture`
Expected: FAIL because order center does not currently trigger checkout recovery.

### Task 2: Implement recovery-safe checkout reconciliation

**Files:**
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`

- [ ] **Step 1: Load existing payment order / attempt / session state inside checkout ensure**
- [ ] **Step 2: Add non-destructive merge rules so advanced payment state is not downgraded**
- [ ] **Step 3: Recreate missing child artifacts using the recovered parent state**

### Task 3: Invoke recovery from portal order-center reads

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`

- [ ] **Step 1: Add a best-effort project order checkout recovery helper**
- [ ] **Step 2: Run recovery before building the order-center response**
- [ ] **Step 3: Keep route availability even if a specific order cannot be repaired**

### Task 4: Verify the closed loop

**Files:**
- Test: `crates/sdkwork-api-app-payment`
- Test: `crates/sdkwork-api-interface-portal`
- Test: `crates/sdkwork-api-interface-admin`

- [ ] **Step 1: Re-run focused tests to verify GREEN**

Run: `cargo test --offline -p sdkwork-api-app-payment --test payment_order_service -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-interface-portal --test portal_order_center -- --nocapture`
Expected: PASS

- [ ] **Step 2: Re-run broader regressions**

Run: `cargo test --offline -p sdkwork-api-interface-portal -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-interface-admin -- --nocapture`
Expected: PASS
