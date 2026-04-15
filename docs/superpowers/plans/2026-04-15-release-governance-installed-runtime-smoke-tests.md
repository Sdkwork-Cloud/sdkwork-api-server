# Release Governance Installed Runtime Smoke Tests Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** ensure the `release-governance` PR workflow executes the direct Unix and Windows installed-runtime smoke helper regressions during preflight.

**Architecture:** extend the release-governance runner with two new direct test lanes for the installed-runtime smoke helpers. Keep blocked-host fallback narrow by validating the helpers' exported parse/plan/evidence contract in-process instead of trying to run a real installed runtime.

**Tech Stack:** Node test runner, release governance preflight runner, installed runtime smoke helper scripts, runtime tooling helpers.

---

## File Map

- Create: `docs/superpowers/specs/2026-04-15-release-governance-installed-runtime-smoke-tests-design.md`
  - capture the remaining installed-runtime smoke execution gap and the recommendation to close both platform lanes together.
- Modify: `scripts/release/tests/release-governance-runner.test.mjs`
  - require the two new plan ids, sequence updates, summary updates, and blocked-host fallback coverage.
- Modify: `scripts/release/run-release-governance-checks.mjs`
  - add the missing preflight plan entries and minimal in-process fallback checks for both smoke helpers.

### Task 1: Write failing runner assertions first

**Files:**
- Modify: `scripts/release/tests/release-governance-runner.test.mjs`

- [ ] **Step 1: Require the new preflight plan ids**

Add plan expectations for:

- `release-unix-installed-runtime-smoke-test`
- `release-windows-installed-runtime-smoke-test`

- [ ] **Step 2: Extend order and aggregate expectations**

Update fixed plan-order assertions, passing-id assertions, and result-order assertions so the runner contract proves both platform smoke lanes execute in release and preflight profiles.

- [ ] **Step 3: Require blocked-host fallback coverage**

Add focused tests that expect both smoke lanes to return fallback success when Node child execution is denied.

- [ ] **Step 4: Run the focused runner test to verify RED**

Run:

```bash
node --test scripts/release/tests/release-governance-runner.test.mjs
```

Expected: FAIL because the runner does not yet expose the two smoke plan ids or fallback handling.

### Task 2: Implement the missing runner coverage

**Files:**
- Modify: `scripts/release/run-release-governance-checks.mjs`

- [ ] **Step 1: Add both installed-runtime smoke plan entries**

Run the direct test files:

- `scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs`
- `scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs`

- [ ] **Step 2: Add in-process fallback verification**

Use the helpers' exported options, plan, and evidence constructors to prove their release-workflow contract remains intact when child Node execution is blocked.

- [ ] **Step 3: Re-run the focused runner test to verify GREEN**

Run:

```bash
node --test scripts/release/tests/release-governance-runner.test.mjs
```

Expected: PASS.

### Task 3: Run focused regression verification

**Files:**
- Modify: none unless verification exposes a real defect

- [ ] **Step 1: Re-run the direct installed-runtime smoke tests**

Run:

```bash
node --test --experimental-test-isolation=none scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs
node --test --experimental-test-isolation=none scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs
```

Expected: PASS for both platform helper suites.

- [ ] **Step 2: Re-run the runner contract**

Run:

```bash
node --test scripts/release/tests/release-governance-runner.test.mjs
```

Expected: PASS.

- [ ] **Step 3: Re-run the actual release-governance preflight command**

Run:

```bash
node scripts/release/run-release-governance-checks.mjs --profile preflight --format json
```

Expected: JSON summary with `"ok": true`.
