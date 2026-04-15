# Single-Node Commercial Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** restore and harden the single-node commercial deployment slice so the product runtime, portal identity path, gateway admission path, and billing/audit evidence path are commercially usable.

**Architecture:** start from the existing integrated product runtime and recover a trustworthy workspace verification baseline first. Then use failing tests and focused runtime checks to close contract drift across portal, gateway, billing, and managed runtime tooling without broad refactors.

**Tech Stack:** Rust workspace crates and services, Axum, SQLx/SQLite, managed PowerShell and shell runtime tooling, React frontends only where directly implicated by runtime closure.

---

## File Map

- Modify: `crates/sdkwork-api-interface-portal/tests/portal_commerce/support.rs`
  - restore portal commerce fixture compatibility with the canonical gateway request context contract.
- Modify: `docs/superpowers/specs/2026-04-14-single-node-commercial-closure-design.md`
  - capture the approved first-phase design for this closure effort.
- Modify: `docs/superpowers/plans/2026-04-14-single-node-commercial-closure.md`
  - track the execution plan for the first-phase closure effort.
- Potential follow-up files after baseline restoration:
  - `crates/sdkwork-api-interface-portal/tests/**`
  - `crates/sdkwork-api-interface-http/tests/**`
  - `services/router-product-service/**`
  - `bin/*.ps1`
  - `bin/*.sh`
  - `scripts/*.mjs`

### Task 1: Restore the first failing workspace verification barrier

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_commerce/support.rs`
- Test: `crates/sdkwork-api-interface-portal/tests/portal_commerce.rs`

- [ ] **Step 1: Reproduce the current failing baseline**

Run:

```bash
cargo test --workspace -q -j 1
```

Expected: FAIL while compiling `sdkwork-api-interface-portal` tests because `GatewayRequestContext` initializers in the portal commerce support helper do not include the canonical subject fields.

- [ ] **Step 2: Align the stale fixture with the current identity contract**

Update the `workspace_request_context()` helper in `crates/sdkwork-api-interface-portal/tests/portal_commerce/support.rs` so it initializes:

- `canonical_tenant_id`
- `canonical_organization_id`
- `canonical_user_id`
- `canonical_api_key_id`

The minimal fix is to set all four fields to `None`, matching the other portal integration tests already updated for the same contract.

- [ ] **Step 3: Run the narrowed failing target**

Run:

```bash
cargo test -p sdkwork-api-interface-portal --test portal_commerce -q
```

Expected: compile succeeds or reveals the next real failure inside the same test target.

### Task 2: Re-establish the workspace Rust verification baseline

**Files:**
- Modify: whichever files the next failure points to
- Test: Rust workspace tests

- [ ] **Step 1: Re-run the workspace baseline**

Run:

```bash
cargo test --workspace -q -j 1
```

Expected: either PASS or fail on the next concrete blocker after Task 1.

- [ ] **Step 2: Fix only the next root-cause blocker**

For each subsequent failure:

- read the failing output completely
- identify the exact contract drift or behavior regression
- add or use the failing test as proof
- implement the smallest fix that restores the contract

- [ ] **Step 3: Repeat until the Rust workspace baseline is trustworthy**

Keep changes focused on commercial closure, not unrelated cleanup.

### Task 3: Verify the single-node commercial main path

**Files:**
- Modify: only if verification reveals defects
- Test: product-runtime and runtime-tooling verification entrypoints

- [ ] **Step 1: Run runtime-focused verification after Rust baseline recovery**

Run the repository-native runtime verification commands that cover managed runtime behavior and product runtime entrypoints.

Initial candidates:

```bash
node --test --experimental-test-isolation=none bin/tests/router-runtime-tooling.test.mjs
node --test scripts/run-router-product-service.test.mjs
node --test scripts/run-router-product.test.mjs
```

Expected: PASS or concrete runtime/tooling defects to address.

- [ ] **Step 2: Fix the highest-severity operational defect**

Prioritize:

- build/install/start/stop breakage
- runtime health endpoint mismatch
- config discovery mismatch
- portal/admin/gateway URL publication mismatch

### Task 4: Verify the commercial evidence chain

**Files:**
- Modify: only if verification reveals defects
- Test: portal/admin/gateway commercial route tests

- [ ] **Step 1: Run targeted commercial integration tests**

Start with the portal/admin/gateway tests already present for:

- portal auth and workspace
- portal API keys and billing
- portal commerce
- gateway compatibility and account admission

- [ ] **Step 2: Fix only evidence-chain defects**

Prioritize fixes where:

- portal can see foreign tenant/project data
- gateway request context drops canonical identity metadata
- billing or audit paths lose traceability

### Task 5: Final verification

**Files:**
- Modify: none

- [ ] **Step 1: Run the full fresh verification set**

At minimum:

```bash
cargo test --workspace -q -j 1
```

Plus any targeted Node/runtime verification added during Tasks 3 and 4.

- [ ] **Step 2: Review the diff against the first-phase goal**

Confirm the changes only improve:

- workspace verification baseline
- single-node runtime closure
- portal/gateway/billing/audit contract integrity

- [ ] **Step 3: Record residual risks explicitly**

If cluster, multi-region, desktop, or capacity work remains, keep it listed as next-phase work rather than pretending first-phase completion covers it.
