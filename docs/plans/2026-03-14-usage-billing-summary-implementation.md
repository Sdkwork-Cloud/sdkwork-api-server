# Usage And Billing Summary Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add authenticated admin summary APIs for usage and billing, then switch the console usage page to consume those summaries.

**Architecture:** Keep storage unchanged and compute summary read models in the application layer from persisted usage records, ledger entries, and quota policies. Expose the summaries through `sdkwork-api-interface-admin`, mirror the contracts in the console type package and admin SDK, and use them in the usage dashboard so aggregation semantics move out of the UI.

**Tech Stack:** Rust, Axum, serde, TypeScript, React, pnpm

---

### Task 1: Add usage summary read models and tests

**Files:**
- Modify: `crates/sdkwork-api-domain-usage/src/lib.rs`
- Modify: `crates/sdkwork-api-app-usage/src/lib.rs`
- Add: `crates/sdkwork-api-app-usage/tests/usage_summary.rs`

**Step 1: Write the failing test**

Create `crates/sdkwork-api-app-usage/tests/usage_summary.rs` with a focused test that proves summary aggregation counts total requests and grouped counts by project, provider, and model.

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-app-usage --test usage_summary -q`

Expected: FAIL because usage summary models or functions do not exist yet.

**Step 3: Write minimal implementation**

Add summary structs to the usage domain crate and a summarization function plus store-backed loader in the usage app crate.

**Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-app-usage --test usage_summary -q`

Expected: PASS

### Task 2: Add billing summary read models and tests

**Files:**
- Modify: `crates/sdkwork-api-domain-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`
- Add: `crates/sdkwork-api-app-billing/tests/billing_summary.rs`

**Step 1: Write the failing test**

Create `crates/sdkwork-api-app-billing/tests/billing_summary.rs` proving the summary computes total entries, units, amount, active quota count, and exhausted project posture.

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-app-billing --test billing_summary -q`

Expected: FAIL because billing summary models or functions do not exist yet.

**Step 3: Write minimal implementation**

Add billing summary structs to the billing domain crate and summary functions in the billing app crate.

**Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-app-billing --test billing_summary -q`

Expected: PASS

### Task 3: Expose summary endpoints through the admin API

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

**Step 1: Write the failing route test**

Add SQLite-backed admin route tests that:

- seed usage records and assert `GET /admin/usage/summary`
- seed ledger entries and quota policies and assert `GET /admin/billing/summary`

**Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`

Expected: FAIL because the routes or handlers do not exist yet.

**Step 3: Write minimal implementation**

Add authenticated handlers and routes wired to the new app-layer summary functions.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`

Expected: PASS

### Task 4: Expose summary contracts to the console and consume them

**Files:**
- Modify: `console/packages/sdkwork-api-types/src/index.ts`
- Modify: `console/packages/sdkwork-api-admin-sdk/src/index.ts`
- Modify: `console/packages/sdkwork-api-usage/src/index.tsx`

**Step 1: Write the failing consumer expectation**

Adjust the usage page to rely on typed summary payloads so TypeScript fails until the SDK and shared types expose the new contracts.

**Step 2: Run typecheck to verify it fails**

Run: `pnpm --dir console -r typecheck`

Expected: FAIL because the new summary SDK functions and types are not defined.

**Step 3: Write minimal implementation**

Add TypeScript summary interfaces, SDK fetch helpers, and update the usage page to use server-computed metrics and grouped summary lists.

**Step 4: Run typecheck and build to verify they pass**

Run: `pnpm --dir console -r typecheck`

Expected: PASS

Run: `pnpm --dir console build`

Expected: PASS

### Task 5: Run full verification and commit

**Files:**
- Modify: repository worktree from previous tasks

**Step 1: Run focused and workspace verification**

Run:

- `cargo test -p sdkwork-api-app-usage --test usage_summary -q`
- `cargo test -p sdkwork-api-app-billing --test billing_summary -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`
- `cargo fmt --all --check`
- `cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`
- `pnpm --dir console build`

Expected: all commands exit `0`

**Step 2: Commit**

```bash
git add docs/plans/2026-03-14-usage-billing-summary-design.md docs/plans/2026-03-14-usage-billing-summary-implementation.md crates/sdkwork-api-domain-usage/src/lib.rs crates/sdkwork-api-app-usage/src/lib.rs crates/sdkwork-api-app-usage/tests/usage_summary.rs crates/sdkwork-api-domain-billing/src/lib.rs crates/sdkwork-api-app-billing/src/lib.rs crates/sdkwork-api-app-billing/tests/billing_summary.rs crates/sdkwork-api-interface-admin/src/lib.rs crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs console/packages/sdkwork-api-types/src/index.ts console/packages/sdkwork-api-admin-sdk/src/index.ts console/packages/sdkwork-api-usage/src/index.tsx
git commit -m "feat: add usage and billing summary admin APIs"
```
