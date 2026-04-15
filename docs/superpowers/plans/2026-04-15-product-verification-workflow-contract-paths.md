# Product Verification Workflow Contract Paths Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** ensure `product-verification` workflow tests fail if the workflow stops watching its contract helper module.

**Architecture:** keep the existing `product-verification.yml` trigger surface intact, then harden the direct workflow test and contract helper so the watched contract path becomes an enforced invariant instead of an incidental YAML detail.

**Tech Stack:** GitHub Actions workflow YAML, Node test runner, workflow contract helper.

---

## File Map

- Create: `docs/superpowers/specs/2026-04-15-product-verification-workflow-contract-paths-design.md`
  - capture the latent regression channel and recommended contract fix.
- Modify: `scripts/product-verification-workflow.test.mjs`
  - add direct assertion coverage and a rejecting fixture.
- Modify: `scripts/product-verification-workflow-contracts.mjs`
  - enforce the contract-helper watched path.

### Task 1: Write the failing workflow test first

**Files:**
- Modify: `scripts/product-verification-workflow.test.mjs`

- [ ] **Step 1: Add the contract-path assertion**

Require the real workflow to contain:

- `scripts/product-verification-workflow-contracts.mjs`

- [ ] **Step 2: Add a rejecting fixture**

Create a workflow fixture that omits `scripts/product-verification-workflow-contracts.mjs` and assert the contract helper rejects it.

- [ ] **Step 3: Run the test to verify RED**

Run:

```bash
node --test scripts/product-verification-workflow.test.mjs
```

Expected: FAIL because the helper does not yet enforce the missing contract path.

### Task 2: Implement the contract fix

**Files:**
- Modify: `scripts/product-verification-workflow-contracts.mjs`

- [ ] **Step 1: Enforce the contract path**

Add an assertion for:

```text
scripts/product-verification-workflow-contracts.mjs
```

with a specific failure message.

- [ ] **Step 2: Re-run the workflow test to verify GREEN**

Run:

```bash
node --test scripts/product-verification-workflow.test.mjs
```

Expected: PASS.

### Task 3: Run focused regression verification

**Files:**
- Modify: none unless verification exposes a real regression

- [ ] **Step 1: Re-run product verification support tests**

Run:

```bash
node --test scripts/check-router-product.test.mjs scripts/build-router-desktop-assets.test.mjs scripts/check-router-docs-safety.test.mjs scripts/check-router-frontend-budgets.test.mjs apps/sdkwork-router-portal/tests/product-entrypoint-scripts.test.mjs
```

Expected: PASS.

- [ ] **Step 2: Inspect the targeted diff**

Run:

```bash
git diff -- scripts/product-verification-workflow.test.mjs scripts/product-verification-workflow-contracts.mjs docs/superpowers/specs/2026-04-15-product-verification-workflow-contract-paths-design.md docs/superpowers/plans/2026-04-15-product-verification-workflow-contract-paths.md
```

Expected: only the product-verification contract-path hardening slice appears.
