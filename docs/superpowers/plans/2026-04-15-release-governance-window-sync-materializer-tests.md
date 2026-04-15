# Release Governance Window And Sync Materializer Tests Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** ensure the `release-governance` PR workflow executes the direct release window snapshot and release sync audit materializer regressions during preflight.

**Architecture:** keep the workflow entrypoint unchanged and extend the release-governance runner so both Git-derived governed artifact materializers are explicit preflight lanes. Use narrow in-process fallbacks on blocked hosts so the runner still verifies the producer contracts without trying to replay the full direct suite inline.

**Tech Stack:** Node test runner, release governance preflight runner, repository-owned release artifact materializers.

---

## File Map

- Create: `docs/superpowers/specs/2026-04-15-release-governance-window-sync-materializer-tests-design.md`
  - capture the watched-but-not-executed Git-derived materializer gap and the recommended fix.
- Modify: `scripts/release/tests/release-governance-runner.test.mjs`
  - require the two new plan ids, sequence expectations, summary expectations, and blocked-host fallback coverage.
- Modify: `scripts/release/run-release-governance-checks.mjs`
  - add the missing preflight plan entries and minimal in-process fallback handling.

### Task 1: Write failing runner assertions first

**Files:**
- Modify: `scripts/release/tests/release-governance-runner.test.mjs`

- [ ] **Step 1: Require the new preflight plan ids**

Add plan expectations for:

- `release-window-snapshot-materializer-test`
- `release-sync-audit-materializer-test`

- [ ] **Step 2: Extend aggregate summary and result-order expectations**

Update the fixed plan order, passing-id order, and status-order assertions so the runner contract proves both materializer lanes execute in normal preflight and full release profiles.

- [ ] **Step 3: Require blocked-host fallback coverage**

Add focused tests that expect each new plan id to return fallback success when Node child execution is denied.

- [ ] **Step 4: Run the focused test to verify RED**

Run:

```bash
node --test scripts/release/tests/release-governance-runner.test.mjs
```

Expected: FAIL because the runner does not yet expose the two plan ids or their fallback handling.

### Task 2: Implement the missing runner coverage

**Files:**
- Modify: `scripts/release/run-release-governance-checks.mjs`

- [ ] **Step 1: Add the two preflight test plan entries**

Run the direct test files:

- `scripts/release/tests/materialize-release-window-snapshot.test.mjs`
- `scripts/release/tests/materialize-release-sync-audit.test.mjs`

- [ ] **Step 2: Add minimal in-process fallback handling**

Use existing producer-input helpers and validators to prove the governed snapshot and sync-audit materializers still produce well-shaped artifacts with the expected governed source kinds when Node child execution is blocked.

- [ ] **Step 3: Re-run the focused runner test to verify GREEN**

Run:

```bash
node --test scripts/release/tests/release-governance-runner.test.mjs
```

Expected: PASS.

### Task 3: Run focused regression verification

**Files:**
- Modify: none unless verification exposes a real defect

- [ ] **Step 1: Re-run the direct materializer suite plus runner contract**

Run:

```bash
node --test scripts/release/tests/materialize-release-window-snapshot.test.mjs scripts/release/tests/materialize-release-sync-audit.test.mjs scripts/release/tests/release-governance-runner.test.mjs
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
git diff -- docs/superpowers/specs/2026-04-15-release-governance-window-sync-materializer-tests-design.md docs/superpowers/plans/2026-04-15-release-governance-window-sync-materializer-tests.md scripts/release/tests/release-governance-runner.test.mjs scripts/release/run-release-governance-checks.mjs
```

Expected: only the release-governance window and sync materializer preflight slice appears.
