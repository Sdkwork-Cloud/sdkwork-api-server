# Payment Over-Capture Governance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Keep local payment orders, refund ceilings, and fulfillment safe when provider settlement callbacks report captured money above the local payable amount.

**Architecture:** Compute an accepted local capture amount for each settled callback, capped by the order payable amount and existing accepted sale rows. Persist the accepted amount in sale transactions and aggregate order state, while writing reconciliation evidence whenever provider-reported capture exceeds what the local system can safely recognize.

**Tech Stack:** Rust, sqlx, SQLite, Axum, cargo test

---

### Task 1: Lock over-capture behavior with failing tests

**Files:**
- Modify: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/payment_callbacks.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`

- [ ] **Step 1: Write the failing app regression for first-capture overpay**

```rust
#[tokio::test]
async fn settled_overcapture_caps_local_capture_and_records_reconciliation() {}
```

- [ ] **Step 2: Write the failing app regression for same-transaction overpay replay**

```rust
#[tokio::test]
async fn same_sale_replay_overcapture_caps_aggregate_capture_without_duplicate_fulfillment() {}
```

- [ ] **Step 3: Write the failing HTTP regression for capped callback projection**

```rust
#[tokio::test]
async fn payment_callback_route_caps_overcapture_to_payable_amount() {}
```

- [ ] **Step 4: Write the failing portal regression for capped refundable amount**

```rust
#[tokio::test]
async fn portal_order_center_caps_refundable_amount_after_overcapture() {}
```

- [ ] **Step 5: Run targeted tests to verify they fail**

Run: `cargo test --offline -p sdkwork-api-app-payment settled_overcapture_caps_local_capture_and_records_reconciliation -- --nocapture`
Expected: FAIL because the first settled overpay is still accepted above the safe local payable amount.

Run: `cargo test --offline -p sdkwork-api-app-payment same_sale_replay_overcapture_caps_aggregate_capture_without_duplicate_fulfillment -- --nocapture`
Expected: FAIL because same-transaction replay can still push local capture above payable.

Run: `cargo test --offline -p sdkwork-api-interface-http payment_callback_route_caps_overcapture_to_payable_amount --test payment_callbacks -- --nocapture`
Expected: FAIL because callback projection still exposes the uncapped local capture amount.

Run: `cargo test --offline -p sdkwork-api-interface-portal portal_order_center_caps_refundable_amount_after_overcapture --test portal_order_center -- --nocapture`
Expected: FAIL because order-center refund capacity is not yet explicitly governed for over-capture.

### Task 2: Implement accepted-capture capping and reconciliation evidence

**Files:**
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`

- [ ] **Step 1: Add helpers that compute accepted local capture against payable and existing sale rows**
- [ ] **Step 2: Cap new sale rows and same-transaction replays to the accepted local amount**
- [ ] **Step 3: Persist mismatch reconciliation evidence when provider-reported capture exceeds the accepted local amount**
- [ ] **Step 4: Keep aggregated `captured_amount_minor` and refund ceiling bounded by accepted local capture**
- [ ] **Step 5: Preserve single fulfillment and single account-grant side effects**

- [ ] **Step 6: Re-run targeted app regressions**

Run: `cargo test --offline -p sdkwork-api-app-payment settled_overcapture_caps_local_capture_and_records_reconciliation -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-app-payment same_sale_replay_overcapture_caps_aggregate_capture_without_duplicate_fulfillment -- --nocapture`
Expected: PASS

### Task 3: Verify projected state through HTTP and portal surfaces

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/payment_callbacks.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`

- [ ] **Step 1: Ensure callback results expose capped `captured_amount_minor`**
- [ ] **Step 2: Ensure portal order center exposes capped `captured_amount_minor` and refundable amount**

- [ ] **Step 3: Re-run targeted interface regressions**

Run: `cargo test --offline -p sdkwork-api-interface-http payment_callback_route_caps_overcapture_to_payable_amount --test payment_callbacks -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-interface-portal portal_order_center_caps_refundable_amount_after_overcapture --test portal_order_center -- --nocapture`
Expected: PASS

### Task 4: Re-verify the affected payment slice

**Files:**
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Test: `crates/sdkwork-api-interface-http/tests/payment_callbacks.rs`
- Test: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`
- Test: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`

- [ ] **Step 1: Run payment app tests**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS

- [ ] **Step 2: Run HTTP callback tests**

Run: `cargo test --offline -p sdkwork-api-interface-http --test payment_callbacks -- --nocapture`
Expected: PASS

- [ ] **Step 3: Run portal order-center tests**

Run: `cargo test --offline -p sdkwork-api-interface-portal --test portal_order_center -- --nocapture`
Expected: PASS

- [ ] **Step 4: Run admin payment tests**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: PASS
