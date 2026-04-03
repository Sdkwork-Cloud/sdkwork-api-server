# Canonical Account Subject Resolution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** add the canonical lookup path from `GatewayAuthSubject` to the active primary `ai_account` so the commercial billing kernel has a real payable-account resolver before hold and settlement mutations land.

**Architecture:** keep the rule out of the HTTP interface and place it in `sdkwork-api-app-billing`. Add a focused `AccountKernelStore` owner-scope query, implement it in SQLite, and use TDD to build an app-billing resolver that returns the active primary account or fails closed on inactive accounts.

**Tech Stack:** Rust, anyhow, async-trait, sqlx, SQLite, cargo test

---

### Task 1: Add failing tests for canonical payable-account resolution

**Files:**
- Modify: `crates/sdkwork-api-app-billing/tests/account_kernel_service.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/account_kernel_contract.rs`

- [ ] **Step 1: Add a failing SQLite-backed storage test for `find_account_record_by_owner(...)`**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-storage-sqlite --test account_kernel_contract -- --nocapture` and verify it fails**
- [ ] **Step 3: Add a failing app-billing test that resolves an active primary account from `GatewayAuthSubject`**
- [ ] **Step 4: Add focused app-billing tests for missing and inactive primary accounts**
- [ ] **Step 5: Run `cargo test -p sdkwork-api-app-billing --test account_kernel_service -- --nocapture` and verify failure**

### Task 2: Add the owner-scope account-kernel query

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`

- [ ] **Step 1: Add `find_account_record_by_owner(tenant_id, organization_id, user_id, account_type)` to `AccountKernelStore`**
- [ ] **Step 2: Implement the real indexed SQLite query against `ai_account`**
- [ ] **Step 3: Re-run the storage-focused test and confirm green**

### Task 3: Implement app-billing payable-account resolution

**Files:**
- Modify: `crates/sdkwork-api-app-billing/Cargo.toml`
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-billing/tests/account_kernel_service.rs`

- [ ] **Step 1: Add the identity-domain dependency needed for `GatewayAuthSubject`**
- [ ] **Step 2: Implement `resolve_payable_account_for_gateway_subject(store, subject)`**
- [ ] **Step 3: Ensure the resolver returns `None` for missing primary accounts**
- [ ] **Step 4: Ensure the resolver errors for non-active primary accounts**
- [ ] **Step 5: Re-run the app-billing focused test and confirm green**

### Task 4: Verify the commercial-kernel baseline and update audit status

**Files:**
- Modify: `docs/superpowers/specs/2026-04-03-router-implementation-audit-and-upgrade-plan.md`

- [ ] **Step 1: Run `cargo test -p sdkwork-api-storage-sqlite --test account_kernel_contract -- --nocapture`**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-app-billing -- --nocapture`**
- [ ] **Step 3: Update the audit doc to record that canonical subject-to-account resolution now exists and that transactional hold-settle orchestration remains the next blocker**
- [ ] **Step 4: Review `git diff` and keep the slice limited to resolution behavior and audit updates**
