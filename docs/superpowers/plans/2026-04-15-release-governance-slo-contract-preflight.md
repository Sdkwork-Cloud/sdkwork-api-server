# Release Governance SLO Contract Preflight Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** ensure normal `release-governance` preflight validates the governed SLO architecture baselines instead of checking them only in fallback mode.

**Architecture:** add a dedicated SLO contracts test to the release-governance runner sequence, then expand the PR workflow path filter and workflow contract assertions so architecture-baseline edits always trigger preflight governance.

**Tech Stack:** GitHub Actions workflow YAML, Node test runner, release governance contract helpers.

---

## File Map

- Create: `docs/superpowers/specs/2026-04-15-release-governance-slo-contract-preflight-design.md`
  - capture the fallback-only governance gap and recommended fix.
- Create: `scripts/release/tests/release-slo-governance-contracts.test.mjs`
  - make SLO contract baselines executable in normal CI.
- Modify: `scripts/release/tests/release-governance-runner.test.mjs`
  - expect the new plan id in normal and preflight sequences.
- Modify: `scripts/release/run-release-governance-checks.mjs`
  - add the new contracts-test plan and fallback branch.
- Modify: `.github/workflows/release-governance.yml`
  - watch the two governed SLO architecture baseline docs.
- Modify: `scripts/release-governance-workflow-contracts.mjs`
  - require those doc paths in the workflow contract.
- Modify: `scripts/release-governance-workflow.test.mjs`
  - assert the real workflow and a rejecting fixture cover those docs.

### Task 1: Write the failing tests first

**Files:**
- Create: `scripts/release/tests/release-slo-governance-contracts.test.mjs`
- Modify: `scripts/release/tests/release-governance-runner.test.mjs`
- Modify: `scripts/release-governance-workflow.test.mjs`

- [ ] **Step 1: Add the new SLO contracts test file**

Call `assertSloGovernanceContracts({ repoRoot })` directly.

- [ ] **Step 2: Tighten runner expectations**

Require a new `release-slo-governance-contracts-test` plan in both the full and preflight sequences.

- [ ] **Step 3: Tighten workflow expectations**

Require the workflow to watch:

- `docs/架构/135-可观测性与SLO治理设计-2026-04-07.md`
- `docs/架构/143-全局架构对齐与收口计划-2026-04-08.md`

and add a rejecting fixture that omits them.

- [ ] **Step 4: Run the tests to verify RED**

Run:

```bash
node --test scripts/release-governance-workflow.test.mjs scripts/release/tests/release-governance-runner.test.mjs scripts/release/tests/release-slo-governance-contracts.test.mjs
```

Expected: FAIL because the runner and workflow do not yet enforce the new contract path.

### Task 2: Implement the normal-path contract fix

**Files:**
- Modify: `scripts/release/run-release-governance-checks.mjs`
- Modify: `.github/workflows/release-governance.yml`
- Modify: `scripts/release-governance-workflow-contracts.mjs`

- [ ] **Step 1: Add the contracts-test plan**

Run:

```text
node --test --experimental-test-isolation=none scripts/release/tests/release-slo-governance-contracts.test.mjs
```

and wire fallback to `assertSloGovernanceContracts`.

- [ ] **Step 2: Expand workflow trigger coverage**

Add the two governed architecture baseline docs to `pull_request.paths`.

- [ ] **Step 3: Re-run the tests to verify GREEN**

Run:

```bash
node --test scripts/release-governance-workflow.test.mjs scripts/release/tests/release-governance-runner.test.mjs scripts/release/tests/release-slo-governance-contracts.test.mjs
```

Expected: PASS.

### Task 3: Run focused regression verification

**Files:**
- Modify: none unless verification exposes a real regression

- [ ] **Step 1: Re-run the actual preflight governance script**

Run:

```bash
node scripts/release/run-release-governance-checks.mjs --profile preflight --format json
```

Expected: JSON output with `"ok": true`.

- [ ] **Step 2: Inspect the targeted diff**

Run:

```bash
git diff -- .github/workflows/release-governance.yml scripts/release-governance-workflow-contracts.mjs scripts/release-governance-workflow.test.mjs scripts/release/run-release-governance-checks.mjs scripts/release/tests/release-governance-runner.test.mjs scripts/release/tests/release-slo-governance-contracts.test.mjs docs/superpowers/specs/2026-04-15-release-governance-slo-contract-preflight-design.md docs/superpowers/plans/2026-04-15-release-governance-slo-contract-preflight.md
```

Expected: only the SLO contract preflight hardening slice appears.
