# Release Governance Runner Workflow Step Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** make the `release-governance` workflow directly execute `release-governance-runner.test.mjs` as a dedicated step before `preflight`.

**Architecture:** keep the runner self-test outside `run-release-governance-checks.mjs` to avoid recursion, and enforce the workflow contract through the existing workflow test plus contract helper. The workflow remains a two-layer gate: explicit runner self-test first, release-governance preflight second.

**Tech Stack:** GitHub Actions workflow YAML, Node test runner, release governance workflow contract helper.

---

## File Map

- Create: `docs/superpowers/specs/2026-04-15-release-governance-runner-workflow-step-design.md`
  - record why the runner self-test should be executed by the workflow but remain outside `preflight`.
- Modify: `scripts/release-governance-workflow.test.mjs`
  - require the dedicated runner self-test workflow step and reject workflows that omit it.
- Modify: `scripts/release-governance-workflow-contracts.mjs`
  - codify the dedicated runner self-test step as part of the workflow contract.
- Modify: `.github/workflows/release-governance.yml`
  - add the dedicated runner self-test step before the `preflight` command.

### Task 1: Write failing workflow assertions first

**Files:**
- Modify: `scripts/release-governance-workflow.test.mjs`
- Modify: `scripts/release-governance-workflow-contracts.mjs`

- [ ] **Step 1: Require the dedicated runner self-test step in the workflow surface test**

Add a positive assertion that the workflow contains:

```bash
node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs
```

- [ ] **Step 2: Add a negative contract fixture**

Create or extend a fixture test that expects the contract helper to reject workflows that omit the dedicated runner self-test step.

- [ ] **Step 3: Run the focused workflow contract suite to verify RED**

Run:

```bash
node --test scripts/release-governance-workflow.test.mjs
```

Expected: FAIL because the workflow does not yet execute the runner self-test.

### Task 2: Implement the workflow contract

**Files:**
- Modify: `.github/workflows/release-governance.yml`
- Modify: `scripts/release-governance-workflow-contracts.mjs`

- [ ] **Step 1: Add the explicit workflow step**

Insert a dedicated step before the `preflight` command that runs:

```bash
node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs
```

- [ ] **Step 2: Update the contract helper**

Require that the workflow contains the dedicated runner self-test step so the contract helper and repository test stay aligned.

- [ ] **Step 3: Re-run the focused workflow suite to verify GREEN**

Run:

```bash
node --test scripts/release-governance-workflow.test.mjs
```

Expected: PASS.

### Task 3: Run verification

**Files:**
- Modify: none unless verification exposes a real defect

- [ ] **Step 1: Re-run the runner self-test directly**

Run:

```bash
node --test scripts/release/tests/release-governance-runner.test.mjs
```

Expected: PASS.

- [ ] **Step 2: Re-run the workflow contract suite**

Run:

```bash
node --test scripts/release-governance-workflow.test.mjs
```

Expected: PASS.

- [ ] **Step 3: Inspect the targeted diff**

Run:

```bash
git diff -- docs/superpowers/specs/2026-04-15-release-governance-runner-workflow-step-design.md docs/superpowers/plans/2026-04-15-release-governance-runner-workflow-step.md scripts/release-governance-workflow.test.mjs scripts/release-governance-workflow-contracts.mjs .github/workflows/release-governance.yml
```

Expected: only the runner workflow-step slice appears.
