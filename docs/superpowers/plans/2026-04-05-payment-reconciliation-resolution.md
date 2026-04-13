# Payment Reconciliation Resolution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let admin operators close payment reconciliation anomalies with durable resolution timestamps while preserving existing payment audit evidence.

**Architecture:** Extend reconciliation records with `updated_at_ms`, preserve that field through storage round-trips, and add an authenticated admin resolve endpoint that updates `match_status` to `resolved` using the existing reconciliation line as the lifecycle object.

**Tech Stack:** Rust, sqlx, SQLite, Postgres contract tests, Axum, cargo test

---

### Task 1: Lock schema and admin resolution behavior with failing tests

**Files:**
- Modify: `crates/sdkwork-api-storage-sqlite/tests/payment_schema.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/payment_schema_contract.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/payment_store.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`

- [ ] **Step 1: Add failing schema expectations for reconciliation `updated_at_ms`**
- [ ] **Step 2: Add failing round-trip expectations for reconciliation `updated_at_ms`**
- [ ] **Step 3: Add failing admin resolve endpoint regression**

- [ ] **Step 4: Run targeted tests to verify they fail**

Run: `cargo test --offline -p sdkwork-api-storage-sqlite --test payment_schema -- --nocapture`
Expected: FAIL because reconciliation schema does not yet declare `updated_at_ms`.

Run: `cargo test --offline -p sdkwork-api-storage-sqlite --test payment_store -- --nocapture`
Expected: FAIL because reconciliation round-trip does not yet preserve `updated_at_ms`.

Run: `cargo test --offline -p sdkwork-api-storage-postgres --test payment_schema_contract -- --nocapture`
Expected: FAIL because postgres payment migrations do not yet declare `updated_at_ms` for reconciliation lines.

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: FAIL because the admin router does not yet expose reconciliation resolution.

### Task 2: Implement reconciliation resolution model and persistence

**Files:**
- Modify: `crates/sdkwork-api-domain-payment/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`

- [ ] **Step 1: Add `updated_at_ms` to `ReconciliationMatchSummaryRecord`**
- [ ] **Step 2: Persist and decode reconciliation `updated_at_ms` in sqlite**
- [ ] **Step 3: Persist and decode reconciliation `updated_at_ms` in postgres**
- [ ] **Step 4: Default new reconciliation rows so `updated_at_ms` tracks creation until resolved**

- [ ] **Step 5: Re-run targeted schema/store tests**

Run: `cargo test --offline -p sdkwork-api-storage-sqlite --test payment_schema -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-storage-sqlite --test payment_store -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-storage-postgres --test payment_schema_contract -- --nocapture`
Expected: PASS

### Task 3: Implement admin reconciliation resolve endpoint

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`

- [ ] **Step 1: Add resolve request DTO and route**
- [ ] **Step 2: Load reconciliation line by id using the existing store abstraction**
- [ ] **Step 3: Update unresolved lines to `resolved` with `updated_at_ms`**
- [ ] **Step 4: Keep repeated resolve requests idempotent**

- [ ] **Step 5: Re-run targeted admin test**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: PASS

### Task 4: Re-verify the affected payment/admin slice

**Files:**
- Test: `crates/sdkwork-api-storage-sqlite/tests/payment_schema.rs`
- Test: `crates/sdkwork-api-storage-sqlite/tests/payment_store.rs`
- Test: `crates/sdkwork-api-storage-postgres/tests/payment_schema_contract.rs`
- Test: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`

- [ ] **Step 1: Run sqlite payment schema tests**

Run: `cargo test --offline -p sdkwork-api-storage-sqlite --test payment_schema -- --nocapture`
Expected: PASS

- [ ] **Step 2: Run sqlite payment store tests**

Run: `cargo test --offline -p sdkwork-api-storage-sqlite --test payment_store -- --nocapture`
Expected: PASS

- [ ] **Step 3: Run postgres payment schema contract tests**

Run: `cargo test --offline -p sdkwork-api-storage-postgres --test payment_schema_contract -- --nocapture`
Expected: PASS

- [ ] **Step 4: Run admin payment tests**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: PASS

- [ ] **Step 5: Run payment app regression tests**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS
