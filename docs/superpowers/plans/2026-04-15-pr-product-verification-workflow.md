# PR Product Verification Workflow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** add a dedicated pull-request product verification workflow that runs the existing governed product verification gate before release time.

**Architecture:** keep `check-router-product.mjs` as the behavioral source of truth, then wrap it in a standalone `product-verification.yml` workflow that mirrors release semantics for frozen installs, strict frontend install mode, and `cargo-audit` availability. Protect the workflow with a dedicated contract helper and workflow test.

**Tech Stack:** GitHub Actions workflow YAML, Node test runner, workflow contract helper, product verification scripts.

---

## File Map

- Create: `docs/superpowers/specs/2026-04-15-pr-product-verification-workflow-design.md`
  - record the PR-time enforcement decision and workflow boundary.
- Create: `.github/workflows/product-verification.yml`
  - add the dedicated PR workflow.
- Create: `scripts/product-verification-workflow-contracts.mjs`
  - assert the required workflow structure and gate wiring.
- Create: `scripts/product-verification-workflow.test.mjs`
  - provide direct workflow assertions plus contract-helper rejection tests.

### Task 1: Write the failing workflow tests first

**Files:**
- Create: `scripts/product-verification-workflow-contracts.mjs`
- Create: `scripts/product-verification-workflow.test.mjs`

- [ ] **Step 1: Add direct workflow assertions**

Require the new workflow to contain:

- `pull_request`
- `workflow_dispatch`
- explicit frozen installs for admin and portal
- `SDKWORK_STRICT_FRONTEND_INSTALLS: '1'`
- `taiki-e/install-action@cargo-audit`
- `node scripts/check-router-product.mjs`

- [ ] **Step 2: Add at least one rejecting contract fixture**

Create a minimal fixture workflow missing strict mode or frozen installs and assert the contract helper rejects it.

- [ ] **Step 3: Run the workflow test to verify RED**

Run:

```bash
node --test scripts/product-verification-workflow.test.mjs
```

Expected: FAIL because the workflow does not yet exist.

### Task 2: Implement the PR workflow

**Files:**
- Create: `.github/workflows/product-verification.yml`

- [ ] **Step 1: Add the workflow triggers and path filters**

Include `pull_request` and `workflow_dispatch`. Cover the product apps, docs, Rust workspace code, scripts, and this workflow/test surface.

- [ ] **Step 2: Add the governed job steps**

Mirror release semantics:

```yaml
- setup pnpm
- setup Node.js 22
- install Rust toolchain
- cache Rust dependencies
- install cargo-audit
- pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
- pnpm --dir apps/sdkwork-router-portal install --frozen-lockfile
- node --test scripts/product-verification-workflow.test.mjs ...
- env: SDKWORK_STRICT_FRONTEND_INSTALLS: '1'
  run: node scripts/check-router-product.mjs
```

- [ ] **Step 3: Re-run the workflow test to verify GREEN**

Run:

```bash
node --test scripts/product-verification-workflow.test.mjs
```

Expected: PASS.

### Task 3: Run focused regression verification

**Files:**
- Modify: none unless verification exposes a real defect

- [ ] **Step 1: Re-run product verification support tests**

Run:

```bash
node --test scripts/check-router-product.test.mjs scripts/build-router-desktop-assets.test.mjs scripts/check-router-docs-safety.test.mjs scripts/check-router-frontend-budgets.test.mjs apps/sdkwork-router-portal/tests/product-entrypoint-scripts.test.mjs
```

Expected: PASS.

- [ ] **Step 2: Re-run the dedicated workflow test**

Run:

```bash
node --test scripts/product-verification-workflow.test.mjs
```

Expected: PASS.

- [ ] **Step 3: Inspect the targeted diff**

Run:

```bash
git diff -- .github/workflows/product-verification.yml scripts/product-verification-workflow-contracts.mjs scripts/product-verification-workflow.test.mjs docs/superpowers/specs/2026-04-15-pr-product-verification-workflow-design.md docs/superpowers/plans/2026-04-15-pr-product-verification-workflow.md
```

Expected: only the PR product verification workflow slice appears.
