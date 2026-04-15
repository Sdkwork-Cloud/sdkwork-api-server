# Release Product Verification Frozen Installs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** make the release `product-verification` gate deterministic by explicitly installing required frontend workspaces with `--frozen-lockfile` before `check-router-product.mjs` runs.

**Architecture:** keep the product verification script unchanged, but stop relying on its implicit auto-install path during release CI. Add a dedicated frozen install step to `product-verification` and lock the behavior in with workflow contract tests.

**Tech Stack:** GitHub Actions workflow YAML, Node test runner, release workflow contract helper, product verification script.

---

## File Map

- Create: `docs/superpowers/specs/2026-04-15-release-product-verification-frozen-installs-design.md`
  - capture the reproducibility problem, options, and recommended release-only fix.
- Modify: `.github/workflows/release.yml`
  - add a frozen install step for admin and portal before `Run release product verification`.
- Modify: `scripts/release/release-workflow-contracts.mjs`
  - require the new install step and its ordering.
- Modify: `scripts/release/tests/release-workflow.test.mjs`
  - add red-green coverage and a rejecting fixture for missing frozen install wiring.

### Task 1: Add failing workflow coverage first

**Files:**
- Modify: `scripts/release/tests/release-workflow.test.mjs`

- [ ] **Step 1: Write the failing release workflow assertion**

Require that `product-verification` contains:

```js
/product-verification:[\\s\\S]*?Install product verification workspace dependencies[\\s\\S]*?pnpm --dir apps\\/sdkwork-router-admin install --frozen-lockfile[\\s\\S]*?pnpm --dir apps\\/sdkwork-router-portal install --frozen-lockfile[\\s\\S]*?Run release product verification[\\s\\S]*?node scripts\\/check-router-product\\.mjs/
```

- [ ] **Step 2: Add a failing contract rejection test**

Create a fixture that removes the frozen install step and assert the contract helper rejects it with a product-verification frozen-install error.

- [ ] **Step 3: Run the targeted workflow suite to verify RED**

Run:

```bash
node --test scripts/release/tests/release-workflow.test.mjs
```

Expected: FAIL because the current `product-verification` job lacks the explicit frozen install step and the helper does not yet enforce it.

### Task 2: Land the workflow and contract change

**Files:**
- Modify: `.github/workflows/release.yml`
- Modify: `scripts/release/release-workflow-contracts.mjs`

- [ ] **Step 1: Update the contract helper**

Require the explicit frozen install step and its order before `Run release product verification`.

- [ ] **Step 2: Update the real workflow**

Add:

```yaml
      - name: Install product verification workspace dependencies
        shell: bash
        run: |
          pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
          pnpm --dir apps/sdkwork-router-portal install --frozen-lockfile
```

before `Run release product verification`.

- [ ] **Step 3: Re-run the targeted workflow suite to verify GREEN**

Run:

```bash
node --test scripts/release/tests/release-workflow.test.mjs
```

Expected: PASS.

### Task 3: Run focused regression verification

**Files:**
- Modify: none unless verification exposes a real defect

- [ ] **Step 1: Re-run the product verification script tests**

Run:

```bash
node --test scripts/check-router-product.test.mjs apps/sdkwork-router-portal/tests/product-entrypoint-scripts.test.mjs
```

Expected: PASS.

- [ ] **Step 2: Re-run the docs safety regression test**

Run:

```bash
node --test scripts/check-router-docs-safety.test.mjs
```

Expected: PASS.

- [ ] **Step 3: Inspect the targeted diff**

Run:

```bash
git diff -- .github/workflows/release.yml scripts/release/release-workflow-contracts.mjs scripts/release/tests/release-workflow.test.mjs docs/superpowers/specs/2026-04-15-release-product-verification-frozen-installs-design.md docs/superpowers/plans/2026-04-15-release-product-verification-frozen-installs.md
```

Expected: only the frozen-install release-governance changes appear.
