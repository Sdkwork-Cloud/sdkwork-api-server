# Portal Order Center And Refund Surface Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose canonical payment/refund/account history through portal and admin HTTP routes without breaking existing portal commerce APIs.

**Architecture:** Add a small query and orchestration layer above the current payment kernel so portal can read joined order/payment/refund state and submit refund requests using existing refund safety rules. Keep interface contracts additive and keep admin changes read-only.

**Tech Stack:** Rust, axum, sqlx, cargo test

---

### Task 1: Lock the portal surface with failing tests

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_commerce.rs`

- [ ] **Step 1: Write the failing order-center test**

```rust
#[tokio::test]
async fn portal_order_center_includes_payment_and_refund_state() {}
```

- [ ] **Step 2: Write the failing refund-request test**

```rust
#[tokio::test]
async fn portal_can_request_refund_for_owned_paid_recharge_order() {}
```

- [ ] **Step 3: Write the failing account-history test**

```rust
#[tokio::test]
async fn portal_account_history_shows_grant_and_refund_reversal() {}
```

- [ ] **Step 4: Run the focused portal tests to verify RED**

Run: `cargo test --offline -p sdkwork-api-interface-portal portal_order_center -- --nocapture`
Expected: FAIL because the routes and handlers do not exist yet.

### Task 2: Add portal payment/refund query services

**Files:**
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`

- [ ] **Step 1: Add portal-facing order-center DTOs**
- [ ] **Step 2: Add project order-center query helper**
- [ ] **Step 3: Add portal refund-request helper**
- [ ] **Step 4: Add portal account-history helper**
- [ ] **Step 5: Re-run focused portal tests**

Run: `cargo test --offline -p sdkwork-api-interface-portal portal_order_center -- --nocapture`
Expected: still FAIL, now because the routes are not wired.

### Task 3: Wire portal HTTP routes

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/Cargo.toml`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`

- [ ] **Step 1: Add `sdkwork-api-app-payment` and `sdkwork-api-domain-payment` dependencies if needed**
- [ ] **Step 2: Add request/response DTOs for refund request and account history**
- [ ] **Step 3: Add `/portal/commerce/order-center` route and handler**
- [ ] **Step 4: Add `/portal/commerce/orders/{order_id}/refunds` route and handler**
- [ ] **Step 5: Add `/portal/billing/account-history` route and handler**
- [ ] **Step 6: Re-run portal tests to verify GREEN**

Run: `cargo test --offline -p sdkwork-api-interface-portal --test portal_commerce -- --nocapture`
Expected: PASS

### Task 4: Add admin read-only payment inspection

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/Cargo.toml`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

- [ ] **Step 1: Write failing admin payment-order and refund-order route tests**
- [ ] **Step 2: Add payment dependencies if needed**
- [ ] **Step 3: Wire `/admin/payments/orders`**
- [ ] **Step 4: Wire `/admin/payments/refunds`**
- [ ] **Step 5: Re-run the focused admin tests**

Run: `cargo test --offline -p sdkwork-api-interface-admin sqlite_admin_routes -- --nocapture`
Expected: PASS

### Task 5: Verify the combined regression surface

**Files:**
- Test: `crates/sdkwork-api-interface-portal/tests/portal_commerce.rs`
- Test: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`

- [ ] **Step 1: Run portal interface tests**

Run: `cargo test --offline -p sdkwork-api-interface-portal -- --nocapture`
Expected: PASS

- [ ] **Step 2: Run admin interface tests**

Run: `cargo test --offline -p sdkwork-api-interface-admin -- --nocapture`
Expected: PASS

- [ ] **Step 3: Re-run payment refund regressions**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS
