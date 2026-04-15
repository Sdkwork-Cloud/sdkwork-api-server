# Product Verification Pnpm Helper Tests Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** ensure the `product-verification` PR workflow executes the watched `pnpm-launch-lib` helper regression test.

**Architecture:** keep the existing product verification gate unchanged, then tighten only the governance test step and its contract assertions so watched pnpm helper tests are executed instead of merely triggering the workflow.

**Tech Stack:** GitHub Actions workflow YAML, Node test runner, workflow contract assertions.

---

## File Map

- Create: `docs/superpowers/specs/2026-04-15-product-verification-pnpm-helper-tests-design.md`
  - capture the watched-test/executed-test mismatch and the recommended fix.
- Modify: `scripts/product-verification-workflow-contracts.mjs`
  - require the Node governance step to execute `scripts/dev/tests/pnpm-launch-lib.test.mjs`.
- Modify: `scripts/product-verification-workflow.test.mjs`
  - extend fixtures so workflows missing the pnpm helper test are rejected.
- Modify: `.github/workflows/product-verification.yml`
  - add the pnpm helper test to the governance Node test step.

### Task 1: Write the failing contract assertions first

**Files:**
- Modify: `scripts/product-verification-workflow-contracts.mjs`
- Modify: `scripts/product-verification-workflow.test.mjs`

- [ ] **Step 1: Tighten the workflow contract**

Require the "Run product governance node tests" command to include:

- `scripts/dev/tests/pnpm-launch-lib.test.mjs`

- [ ] **Step 2: Add a rejecting fixture**

Create or update a fixture-based workflow test that omits the pnpm helper test and assert the contract helper rejects it.

- [ ] **Step 3: Run the workflow test to verify RED**

Run:

```bash
node --test scripts/product-verification-workflow.test.mjs
```

Expected: FAIL because the workflow does not yet execute the pnpm helper test.

### Task 2: Implement the workflow fix

**Files:**
- Modify: `.github/workflows/product-verification.yml`

- [ ] **Step 1: Expand the governance Node test step**

Add `scripts/dev/tests/pnpm-launch-lib.test.mjs` to the `node --test` command in `product-verification.yml`.

- [ ] **Step 2: Re-run the workflow contract test to verify GREEN**

Run:

```bash
node --test scripts/product-verification-workflow.test.mjs
```

Expected: PASS.

### Task 3: Run focused regression verification

**Files:**
- Modify: none unless verification exposes a real defect

- [ ] **Step 1: Re-run the product governance Node suite**

Run:

```bash
node --test scripts/product-verification-workflow.test.mjs scripts/check-router-product.test.mjs scripts/build-router-desktop-assets.test.mjs scripts/check-router-docs-safety.test.mjs scripts/check-router-frontend-budgets.test.mjs scripts/dev/tests/pnpm-launch-lib.test.mjs apps/sdkwork-router-portal/tests/product-entrypoint-scripts.test.mjs
```

Expected: PASS.

- [ ] **Step 2: Inspect the targeted diff**

Run:

```bash
git diff -- .github/workflows/product-verification.yml scripts/product-verification-workflow-contracts.mjs scripts/product-verification-workflow.test.mjs docs/superpowers/specs/2026-04-15-product-verification-pnpm-helper-tests-design.md docs/superpowers/plans/2026-04-15-product-verification-pnpm-helper-tests.md
```

Expected: only the pnpm helper-test execution hardening slice appears.
