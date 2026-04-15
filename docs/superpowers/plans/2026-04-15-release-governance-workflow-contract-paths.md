# Release Governance Workflow Contract Paths Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** close the PR-time governance bypass where `release-governance` workflow contract helper edits do not trigger the `release-governance` workflow.

**Architecture:** keep `scripts/release/run-release-governance-checks.mjs` as the release-governance source of truth, then harden `.github/workflows/release-governance.yml` and its contract helper so the workflow explicitly watches and verifies its own contract module. Protect the change with direct workflow assertions plus a rejecting fixture.

**Tech Stack:** GitHub Actions workflow YAML, Node test runner, workflow contract helper.

---

## File Map

- Create: `docs/superpowers/specs/2026-04-15-release-governance-workflow-contract-paths-design.md`
  - capture the bypass and recommended governance fix.
- Create: `scripts/release-governance-workflow.test.mjs`
  - strengthen workflow coverage with direct assertions and a rejecting fixture.
- Modify: `.github/workflows/release-governance.yml`
  - watch the contract helper path in `pull_request.paths`.
- Modify: `scripts/release-governance-workflow-contracts.mjs`
  - assert the workflow watches the contract helper path.

### Task 1: Write the failing workflow test first

**Files:**
- Modify: `scripts/release-governance-workflow.test.mjs`

- [ ] **Step 1: Add direct workflow assertions**

Require the workflow to contain:

- `pull_request`
- `workflow_dispatch`
- `.github/workflows/release-governance.yml`
- `scripts/release/**`
- `scripts/release-governance-workflow-contracts.mjs`
- `scripts/release-governance-workflow.test.mjs`
- `node scripts/release/run-release-governance-checks.mjs --profile preflight --format json`

- [ ] **Step 2: Add a rejecting contract fixture**

Create a minimal fixture workflow that omits `scripts/release-governance-workflow-contracts.mjs` and assert the helper rejects it.

- [ ] **Step 3: Run the test to verify RED**

Run:

```bash
node --test scripts/release-governance-workflow.test.mjs
```

Expected: FAIL because the live workflow does not yet watch the contract helper path.

### Task 2: Implement the governance fix

**Files:**
- Modify: `.github/workflows/release-governance.yml`
- Modify: `scripts/release-governance-workflow-contracts.mjs`

- [ ] **Step 1: Expand workflow paths**

Add:

```yaml
- 'scripts/release-governance-workflow-contracts.mjs'
```

to the `pull_request.paths` list.

- [ ] **Step 2: Harden the contract helper**

Require the workflow text to match the contract helper path with an explicit failure message.

- [ ] **Step 3: Re-run the workflow test to verify GREEN**

Run:

```bash
node --test scripts/release-governance-workflow.test.mjs
```

Expected: PASS.

### Task 3: Run focused regression verification

**Files:**
- Modify: none unless verification exposes a real regression

- [ ] **Step 1: Re-run the release-governance runner tests**

Run:

```bash
node --test scripts/release/tests/release-governance-runner.test.mjs
```

Expected: PASS.

- [ ] **Step 2: Re-run the actual preflight governance script**

Run:

```bash
node scripts/release/run-release-governance-checks.mjs --profile preflight --format json
```

Expected: JSON output with `"ok": true`.

- [ ] **Step 3: Inspect the targeted diff**

Run:

```bash
git diff -- .github/workflows/release-governance.yml scripts/release-governance-workflow-contracts.mjs scripts/release-governance-workflow.test.mjs docs/superpowers/specs/2026-04-15-release-governance-workflow-contract-paths-design.md docs/superpowers/plans/2026-04-15-release-governance-workflow-contract-paths.md
```

Expected: only the release-governance contract-path hardening slice appears.
