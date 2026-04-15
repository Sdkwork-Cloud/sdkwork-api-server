# Release Governance Bundle Restore Tests Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** ensure the `release-governance` PR workflow executes the direct bundle and restore regression tests during preflight.

**Architecture:** keep the workflow entrypoint unchanged, then extend the release-governance runner so the watched operator recovery chain is explicitly exercised by preflight and still tolerates blocked local child-process execution through minimal in-process fallbacks.

**Tech Stack:** Node test runner, release governance preflight runner, GitHub Actions workflow delegation.

---

## File Map

- Create: `docs/superpowers/specs/2026-04-15-release-governance-bundle-restore-tests-design.md`
  - capture the watched-but-not-executed governance bundle and restore gap.
- Modify: `scripts/release/tests/release-governance-runner.test.mjs`
  - require explicit plan ids, ordering, aggregate summaries, and blocked-host fallbacks for the bundle and restore tests.
- Modify: `scripts/release/run-release-governance-checks.mjs`
  - add the missing preflight plan entries and minimal fallback handling.

### Task 1: Write failing runner assertions first

**Files:**
- Modify: `scripts/release/tests/release-governance-runner.test.mjs`

- [ ] **Step 1: Require the new preflight plan ids**

Add plan expectations for:

- `release-governance-bundle-test`
- `restore-release-governance-latest-test`

- [ ] **Step 2: Extend aggregate summary expectations**

Update the passing-order assertions so the runner contract proves those two lanes execute as part of normal preflight and full release profiles.

- [ ] **Step 3: Require blocked-host fallback coverage**

Add focused tests that expect both plan ids to return fallback success when Node child execution is denied.

- [ ] **Step 4: Run the focused test to verify RED**

Run:

```bash
node --test scripts/release/tests/release-governance-runner.test.mjs
```

Expected: FAIL because the runner does not yet expose those plan ids or fallbacks.

### Task 2: Implement the missing runner coverage

**Files:**
- Modify: `scripts/release/run-release-governance-checks.mjs`

- [ ] **Step 1: Add the preflight test plan entries**

Run the direct test files:

- `scripts/release/tests/materialize-release-governance-bundle.test.mjs`
- `scripts/release/tests/restore-release-governance-latest.test.mjs`

- [ ] **Step 2: Add minimal in-process fallback handling**

Keep the fallback narrow and contract-oriented so blocked local hosts still validate the operator recovery chain without reproducing the full direct test suite inline.

- [ ] **Step 3: Re-run the focused runner test to verify GREEN**

Run:

```bash
node --test scripts/release/tests/release-governance-runner.test.mjs
```

Expected: PASS.

### Task 3: Run focused regression verification

**Files:**
- Modify: none unless verification exposes a real defect

- [ ] **Step 1: Re-run the direct helper suite plus runner contract**

Run:

```bash
node --test scripts/release/tests/materialize-release-governance-bundle.test.mjs scripts/release/tests/restore-release-governance-latest.test.mjs scripts/release/tests/release-governance-runner.test.mjs
```

Expected: PASS.

- [ ] **Step 2: Re-run the actual release-governance preflight command**

Run:

```bash
node scripts/release/run-release-governance-checks.mjs --profile preflight --format json
```

Expected: JSON summary with `"ok": true`.

- [ ] **Step 3: Inspect the targeted diff**

Run:

```bash
git diff -- docs/superpowers/specs/2026-04-15-release-governance-bundle-restore-tests-design.md docs/superpowers/plans/2026-04-15-release-governance-bundle-restore-tests.md scripts/release/tests/release-governance-runner.test.mjs scripts/release/run-release-governance-checks.mjs
```

Expected: only the release-governance bundle/restore preflight slice appears.
