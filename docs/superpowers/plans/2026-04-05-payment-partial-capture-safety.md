# Payment Partial Capture Safety Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prevent partial settlement callbacks from triggering full order fulfillment, full recharge grants, or refund ceilings above the actually captured amount.

**Architecture:** Extend the payment-order model with explicit partial-capture state and a persisted `captured_amount_minor` field. Keep one canonical sale transaction per payment order in this tranche, upgrade it monotonically as callbacks advance from partial to full capture, and drive refund eligibility plus portal projections from captured money instead of payable money.

**Tech Stack:** Rust, sqlx, SQLite, Postgres, Axum, cargo test

---

### Task 1: Lock the regression with failing tests

**Files:**
- Modify: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/payment_callbacks.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`

- [ ] **Step 1: Write the failing app regression for partial capture**

```rust
#[tokio::test]
async fn verified_partial_settlement_keeps_order_unfulfilled_and_limits_refunds() {}
```

- [ ] **Step 2: Write the failing app regression for upgrade from partial to full capture**

```rust
#[tokio::test]
async fn full_capture_replay_upgrades_partial_capture_without_duplicate_sale_side_effects() {}
```

- [ ] **Step 3: Write the failing HTTP regression for partial capture callback projection**

```rust
#[tokio::test]
async fn payment_callback_route_returns_partial_capture_state_for_underpaid_settlement() {}
```

- [ ] **Step 4: Write the failing portal regression for refundable amount from captured money**

```rust
#[tokio::test]
async fn portal_order_center_uses_captured_amount_for_partial_capture_refunds() {}
```

- [ ] **Step 5: Run targeted tests to verify they fail**

Run: `cargo test --offline -p sdkwork-api-app-payment verified_partial_settlement_keeps_order_unfulfilled_and_limits_refunds -- --nocapture`
Expected: FAIL because partial settlement is still treated as full capture.

Run: `cargo test --offline -p sdkwork-api-app-payment full_capture_replay_upgrades_partial_capture_without_duplicate_sale_side_effects -- --nocapture`
Expected: FAIL because the payment order cannot distinguish partial capture from full capture yet.

Run: `cargo test --offline -p sdkwork-api-interface-http payment_callback_route_returns_partial_capture_state_for_underpaid_settlement --test payment_callbacks -- --nocapture`
Expected: FAIL because the callback route reports settled capture as full success.

Run: `cargo test --offline -p sdkwork-api-interface-portal portal_order_center_uses_captured_amount_for_partial_capture_refunds --test portal_order_center -- --nocapture`
Expected: FAIL because refundable amount is still derived from `payable_minor`.

### Task 2: Extend the payment model and storage

**Files:**
- Modify: `crates/sdkwork-api-domain-payment/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/payment_store.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/payment_schema.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/payment_schema_contract.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`

- [ ] **Step 1: Add `PaymentOrderStatus::PartiallyCaptured` and `captured_amount_minor`**
- [ ] **Step 2: Persist the new column in SQLite and Postgres migrations plus decode/insert paths**
- [ ] **Step 3: Add storage round-trip and schema assertions for the new column**

- [ ] **Step 4: Run targeted storage tests**

Run: `cargo test --offline -p sdkwork-api-storage-sqlite --test payment_store -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-storage-sqlite --test payment_schema -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-storage-postgres --test payment_schema_contract -- --nocapture`
Expected: PASS

### Task 3: Implement partial capture progression and refund ceilings

**Files:**
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`

- [ ] **Step 1: Derive partial vs full capture from callback amount**
- [ ] **Step 2: Update payment status and fulfillment status monotically**
- [ ] **Step 3: Persist monotonic `captured_amount_minor`**
- [ ] **Step 4: Update canonical sale transaction amounts on replay upgrades**
- [ ] **Step 5: Gate fulfillment and account grants on full capture only**
- [ ] **Step 6: Base refund support, remaining refundable amount, and refund status on captured money**

- [ ] **Step 7: Re-run targeted app regressions**

Run: `cargo test --offline -p sdkwork-api-app-payment verified_partial_settlement_keeps_order_unfulfilled_and_limits_refunds -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-app-payment full_capture_replay_upgrades_partial_capture_without_duplicate_sale_side_effects -- --nocapture`
Expected: PASS

### Task 4: Expose partial capture through gateway and portal surfaces

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/payment_callbacks.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`

- [ ] **Step 1: Ensure HTTP callback response serializes `partially_captured` for underpaid settlement**
- [ ] **Step 2: Ensure portal order-center projections surface `captured_amount_minor` and partial refund capacity**

- [ ] **Step 3: Re-run targeted interface regressions**

Run: `cargo test --offline -p sdkwork-api-interface-http payment_callback_route_returns_partial_capture_state_for_underpaid_settlement --test payment_callbacks -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-interface-portal portal_order_center_uses_captured_amount_for_partial_capture_refunds --test portal_order_center -- --nocapture`
Expected: PASS

### Task 5: Re-verify the affected payment slice

**Files:**
- Test: `crates/sdkwork-api-domain-payment/src/lib.rs`
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Test: `crates/sdkwork-api-interface-http/tests/payment_callbacks.rs`
- Test: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`
- Test: `crates/sdkwork-api-storage-sqlite/tests/payment_store.rs`
- Test: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`

- [ ] **Step 1: Run domain payment tests**

Run: `cargo test --offline -p sdkwork-api-domain-payment -- --nocapture`
Expected: PASS

- [ ] **Step 2: Run payment app tests**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS

- [ ] **Step 3: Run HTTP payment callback tests**

Run: `cargo test --offline -p sdkwork-api-interface-http --test payment_callbacks -- --nocapture`
Expected: PASS

- [ ] **Step 4: Run portal order-center tests**

Run: `cargo test --offline -p sdkwork-api-interface-portal --test portal_order_center -- --nocapture`
Expected: PASS

- [ ] **Step 5: Run SQLite payment store tests**

Run: `cargo test --offline -p sdkwork-api-storage-sqlite --test payment_store -- --nocapture`
Expected: PASS

- [ ] **Step 6: Run admin payment tests**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: PASS
