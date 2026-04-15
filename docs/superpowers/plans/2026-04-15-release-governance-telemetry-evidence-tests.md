# Release Governance Telemetry Evidence Tests Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** ensure the `release-governance` PR workflow executes the direct telemetry export, telemetry snapshot, and SLO evidence regression tests during preflight.

**Architecture:** keep the workflow entrypoint unchanged and extend the release-governance runner so the governed telemetry evidence chain is explicitly exercised by preflight. Use narrow in-process fallbacks on blocked hosts so the runner still verifies the helper contracts without trying to reproduce every direct test inline.

**Tech Stack:** Node test runner, release governance preflight runner, repository-owned telemetry evidence materializers.

---

## File Map

- Create: `docs/superpowers/specs/2026-04-15-release-governance-telemetry-evidence-tests-design.md`
  - capture the watched-but-not-executed telemetry evidence-chain gap and the recommended fix.
- Modify: `scripts/release/tests/release-governance-runner.test.mjs`
  - require explicit telemetry evidence plan ids, ordering, summary expectations, and blocked-host fallbacks.
- Modify: `scripts/release/run-release-governance-checks.mjs`
  - add the missing preflight plan entries and minimal telemetry fallback handling.

### Task 1: Write failing runner assertions first

**Files:**
- Modify: `scripts/release/tests/release-governance-runner.test.mjs`

- [ ] **Step 1: Require the new preflight plan ids**

Add plan expectations for:

- `release-telemetry-export-test`
- `release-telemetry-snapshot-test`
- `release-slo-evidence-materializer-test`

- [ ] **Step 2: Extend aggregate summary expectations**

Update the passing-order and result-order assertions so the runner contract proves these three lanes execute as part of normal preflight and full release profiles.

- [ ] **Step 3: Require blocked-host fallback coverage**

Add focused tests that expect each new plan id to return fallback success when Node child execution is denied.

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

- `scripts/release/tests/materialize-release-telemetry-export.test.mjs`
- `scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs`
- `scripts/release/tests/materialize-slo-governance-evidence.test.mjs`

- [ ] **Step 2: Add minimal in-process fallback handling**

Use the existing exported helper functions and shape validators so blocked local hosts still prove that the telemetry evidence pipeline stays wired correctly without reimplementing the full direct test suite.

- [ ] **Step 3: Re-run the focused runner test to verify GREEN**

Run:

```bash
node --test scripts/release/tests/release-governance-runner.test.mjs
```

Expected: PASS.

### Task 3: Run focused regression verification

**Files:**
- Modify: none unless verification exposes a real defect

- [ ] **Step 1: Re-run the direct telemetry helper suite plus runner contract**

Run:

```bash
node --test scripts/release/tests/materialize-release-telemetry-export.test.mjs scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs scripts/release/tests/materialize-slo-governance-evidence.test.mjs scripts/release/tests/release-governance-runner.test.mjs
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
git diff -- docs/superpowers/specs/2026-04-15-release-governance-telemetry-evidence-tests-design.md docs/superpowers/plans/2026-04-15-release-governance-telemetry-evidence-tests.md scripts/release/tests/release-governance-runner.test.mjs scripts/release/run-release-governance-checks.mjs
```

Expected: only the release-governance telemetry evidence preflight slice appears.
