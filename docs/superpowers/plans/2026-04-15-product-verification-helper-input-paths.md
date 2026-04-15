# Product Verification Helper Input Paths Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** ensure the `product-verification` pull-request workflow runs when shared desktop/runtime helper scripts used by the product gate change.

**Architecture:** keep `check-router-product.mjs` as the behavioral source of truth, then expand the workflow path filter and contract assertions so helper-module changes in `run-tauri-cli.mjs` and `release/desktop-targets.mjs` cannot bypass PR-time product verification.

**Tech Stack:** GitHub Actions workflow YAML, Node test runner, workflow contract helper.

---

## File Map

- Create: `docs/superpowers/specs/2026-04-15-product-verification-helper-input-paths-design.md`
  - capture the helper-input governance gap and recommended fix.
- Modify: `scripts/product-verification-workflow.test.mjs`
  - assert the helper paths and add a rejecting fixture.
- Modify: `scripts/product-verification-workflow-contracts.mjs`
  - require the helper paths in the workflow contract.
- Modify: `.github/workflows/product-verification.yml`
  - watch the helper files in `pull_request.paths`.

### Task 1: Write the failing workflow test first

**Files:**
- Modify: `scripts/product-verification-workflow.test.mjs`

- [ ] **Step 1: Add direct helper-path assertions**

Require the workflow to contain:

- `scripts/run-tauri-cli.mjs`
- `scripts/release/desktop-targets.mjs`

- [ ] **Step 2: Add a rejecting fixture**

Create a fixture workflow that omits those helper paths and assert the contract helper rejects it.

- [ ] **Step 3: Run the test to verify RED**

Run:

```bash
node --test scripts/product-verification-workflow.test.mjs
```

Expected: FAIL because the workflow contract helper does not yet enforce those helper paths.

### Task 2: Implement the workflow trigger fix

**Files:**
- Modify: `.github/workflows/product-verification.yml`
- Modify: `scripts/product-verification-workflow-contracts.mjs`

- [ ] **Step 1: Expand workflow trigger coverage**

Add:

```text
scripts/run-tauri-cli.mjs
scripts/release/desktop-targets.mjs
```

to the PR path filter and require them in the contract helper.

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
git diff -- .github/workflows/product-verification.yml scripts/product-verification-workflow.test.mjs scripts/product-verification-workflow-contracts.mjs docs/superpowers/specs/2026-04-15-product-verification-helper-input-paths-design.md docs/superpowers/plans/2026-04-15-product-verification-helper-input-paths.md
```

Expected: only the helper-input path hardening slice appears.
