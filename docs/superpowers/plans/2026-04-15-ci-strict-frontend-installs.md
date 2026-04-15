# CI Strict Frontend Installs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** make CI and release verification fail fast instead of auto-installing frontend dependencies when workspace installs are missing or unhealthy.

**Architecture:** move strict frontend install policy into `pnpm-launch-lib.mjs`, keep local repair behavior as the default, and let the release `product-verification` job opt into strict mode explicitly through environment wiring. Both product verification and desktop asset build scripts must use the same helper.

**Tech Stack:** Node test runner, GitHub Actions workflow YAML, shared pnpm launch utilities, product verification scripts.

---

## File Map

- Create: `docs/superpowers/specs/2026-04-15-ci-strict-frontend-installs-design.md`
  - record the shared strict-mode decision and workflow wiring.
- Modify: `scripts/dev/pnpm-launch-lib.mjs`
  - add shared strict-mode helpers and strict failure behavior.
- Modify: `scripts/dev/tests/pnpm-launch-lib.test.mjs`
  - cover strict-mode parsing and failure semantics.
- Modify: `scripts/check-router-product.mjs`
  - replace local policy logic with the shared helper.
- Modify: `scripts/build-router-desktop-assets.mjs`
  - replace local policy logic with the shared helper.
- Modify: `.github/workflows/release.yml`
  - export strict frontend install mode in `product-verification`.
- Modify: `scripts/release/release-workflow-contracts.mjs`
  - require strict-mode env wiring in the product verification job.
- Modify: `scripts/release/tests/release-workflow.test.mjs`
  - add workflow coverage for strict-mode env wiring.

### Task 1: Add failing tests for strict-mode behavior

**Files:**
- Modify: `scripts/dev/tests/pnpm-launch-lib.test.mjs`
- Modify: `scripts/release/tests/release-workflow.test.mjs`

- [ ] **Step 1: Add shared helper red tests**

Require:

- strict mode parses truthy values from `SDKWORK_STRICT_FRONTEND_INSTALLS`
- a non-ready install state in strict mode throws instead of installing

- [ ] **Step 2: Add release workflow red test**

Require `product-verification` to export:

```yaml
env:
  SDKWORK_STRICT_FRONTEND_INSTALLS: '1'
```

before `Run release product verification`.

- [ ] **Step 3: Run the targeted red suite**

Run:

```bash
node --test scripts/dev/tests/pnpm-launch-lib.test.mjs scripts/release/tests/release-workflow.test.mjs
```

Expected: FAIL because the helpers and workflow env wiring do not yet exist.

### Task 2: Implement the shared strict-mode policy

**Files:**
- Modify: `scripts/dev/pnpm-launch-lib.mjs`
- Modify: `scripts/check-router-product.mjs`
- Modify: `scripts/build-router-desktop-assets.mjs`

- [ ] **Step 1: Add shared strict-mode helpers**

Implement shared exports that:

- resolve whether strict mode is enabled
- either throw on non-ready installs in strict mode or run repair installs in default mode

- [ ] **Step 2: Replace duplicated script policy logic**

Update `check-router-product.mjs` and `build-router-desktop-assets.mjs` so they call the shared helper instead of owning duplicate install-repair behavior.

- [ ] **Step 3: Re-run the shared-helper tests**

Run:

```bash
node --test scripts/dev/tests/pnpm-launch-lib.test.mjs
```

Expected: PASS.

### Task 3: Wire strict mode into release governance

**Files:**
- Modify: `.github/workflows/release.yml`
- Modify: `scripts/release/release-workflow-contracts.mjs`
- Modify: `scripts/release/tests/release-workflow.test.mjs`

- [ ] **Step 1: Export strict mode in product verification**

Add:

```yaml
env:
  SDKWORK_STRICT_FRONTEND_INSTALLS: '1'
```

to the `Run release product verification` step.

- [ ] **Step 2: Require it in contract assertions**

Make the workflow helper and tests fail if strict mode env wiring disappears.

- [ ] **Step 3: Re-run the workflow suite**

Run:

```bash
node --test scripts/release/tests/release-workflow.test.mjs
```

Expected: PASS.

### Task 4: Run focused regression verification

**Files:**
- Modify: none unless verification exposes a real defect

- [ ] **Step 1: Re-run product verification script tests**

Run:

```bash
node --test scripts/check-router-product.test.mjs scripts/build-router-desktop-assets.test.mjs
```

Expected: PASS.

- [ ] **Step 2: Re-run docs safety regression test**

Run:

```bash
node --test scripts/check-router-docs-safety.test.mjs
```

Expected: PASS.

- [ ] **Step 3: Inspect the targeted diff**

Run:

```bash
git diff -- scripts/dev/pnpm-launch-lib.mjs scripts/dev/tests/pnpm-launch-lib.test.mjs scripts/check-router-product.mjs scripts/build-router-desktop-assets.mjs .github/workflows/release.yml scripts/release/release-workflow-contracts.mjs scripts/release/tests/release-workflow.test.mjs docs/superpowers/specs/2026-04-15-ci-strict-frontend-installs-design.md docs/superpowers/plans/2026-04-15-ci-strict-frontend-installs.md
```

Expected: only the strict frontend install governance slice appears.
