# Payment Attempt Failover Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Recover failed or expired checkout attempts by creating a replacement payment attempt and session on the next configured active route.

**Architecture:** Add canonical domain/storage support for gateway accounts and channel policies, use them in the payment app to select the next eligible route, and reopen the payment order only for recoverable terminal outcomes with a full audit trail of old and new attempts.

**Tech Stack:** Rust, sqlx, SQLite, Postgres integration tests, cargo test

---

### Task 1: Lock routing configuration storage and failover behavior with failing tests

**Files:**
- Modify: `crates/sdkwork-api-storage-sqlite/tests/payment_store.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`
- Modify: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`

- [ ] **Step 1: Add failing sqlite/postgres round-trip tests for gateway account and channel policy records**
- [ ] **Step 2: Add a failing callback regression for failed-attempt automatic failover**
- [ ] **Step 3: Add a failing callback regression proving canceled outcomes do not auto-failover**
- [ ] **Step 4: Add a failing duplicate-callback regression to keep failover idempotent**

- [ ] **Step 5: Run targeted tests to verify failure**

Run: `cargo test --offline -p sdkwork-api-storage-sqlite --test payment_store -- --nocapture`
Expected: FAIL because payment routing configuration records are not yet modeled or persisted.

Run: `cargo test --offline -p sdkwork-api-storage-postgres --test integration_postgres -- --nocapture`
Expected: FAIL because payment routing configuration records are not yet modeled or persisted.

Run: `cargo test --offline -p sdkwork-api-app-payment --test payment_callback_processing -- --nocapture`
Expected: FAIL because failed/expired callbacks do not yet create replacement attempts.

### Task 2: Implement payment routing configuration records and persistence

**Files:**
- Modify: `crates/sdkwork-api-domain-payment/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`

- [ ] **Step 1: Add `PaymentGatewayAccountRecord` and `PaymentChannelPolicyRecord` domain models**
- [ ] **Step 2: Extend `PaymentKernelStore` with insert/list methods for both record types**
- [ ] **Step 3: Implement sqlite persistence and decode paths**
- [ ] **Step 4: Implement postgres persistence and decode paths**

- [ ] **Step 5: Re-run targeted storage tests**

Run: `cargo test --offline -p sdkwork-api-storage-sqlite --test payment_store -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-storage-postgres --test integration_postgres -- --nocapture`
Expected: PASS

### Task 3: Implement automatic failover in payment callback processing

**Files:**
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`
- Modify: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`

- [ ] **Step 1: Add route selection helpers based on active channel policies and gateway accounts**
- [ ] **Step 2: Create replacement attempt/session ids and records with incremented `attempt_no`**
- [ ] **Step 3: Reopen payment orders only for `failed` and `expired` outcomes when an alternate route exists**
- [ ] **Step 4: Keep `canceled` terminal and keep duplicate callbacks idempotent**

- [ ] **Step 5: Re-run targeted payment callback tests**

Run: `cargo test --offline -p sdkwork-api-app-payment --test payment_callback_processing -- --nocapture`
Expected: PASS

### Task 4: Re-verify the affected payment slice

**Files:**
- Test: `crates/sdkwork-api-storage-sqlite/tests/payment_store.rs`
- Test: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Test: `crates/sdkwork-api-app-payment/tests/payment_order_service.rs`

- [ ] **Step 1: Re-run sqlite payment store tests**

Run: `cargo test --offline -p sdkwork-api-storage-sqlite --test payment_store -- --nocapture`
Expected: PASS

- [ ] **Step 2: Re-run postgres integration tests**

Run: `cargo test --offline -p sdkwork-api-storage-postgres --test integration_postgres -- --nocapture`
Expected: PASS

- [ ] **Step 3: Re-run payment callback processing tests**

Run: `cargo test --offline -p sdkwork-api-app-payment --test payment_callback_processing -- --nocapture`
Expected: PASS

- [ ] **Step 4: Re-run payment order service tests**

Run: `cargo test --offline -p sdkwork-api-app-payment --test payment_order_service -- --nocapture`
Expected: PASS
