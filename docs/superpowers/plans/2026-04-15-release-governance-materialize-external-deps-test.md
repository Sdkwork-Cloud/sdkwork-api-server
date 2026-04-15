# Release Governance Materialize External Deps Test Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** ensure the `release-governance` PR workflow executes the direct `materialize-external-deps` regression test during preflight.

**Architecture:** keep the current release-governance workflow narrow, then add one explicit helper-regression plan entry and strengthen the workflow/runner contract tests so the watched release helper surface is also executed.

**Tech Stack:** GitHub Actions workflow YAML, Node test runner, release governance preflight runner.

---

## File Map

- Create: `docs/superpowers/specs/2026-04-15-release-governance-materialize-external-deps-test-design.md`
  - capture the watched-but-not-executed helper-test gap and the recommended fix.
- Modify: `scripts/release/tests/release-governance-runner.test.mjs`
  - require a preflight plan id and command entry for `materialize-external-deps.test.mjs`.
- Modify: `scripts/release-governance-workflow-contracts.mjs`
  - require the release-governance workflow to execute the materialize-external-deps test through the preflight runner contract surface.
- Modify: `scripts/release-governance-workflow.test.mjs`
  - extend workflow assertions or fixtures if needed so the contract remains visible at the workflow level.
- Modify: `scripts/release/run-release-governance-checks.mjs`
  - add the missing preflight plan entry and fallback handling.

### Task 1: Write the failing runner/workflow assertions first

**Files:**
- Modify: `scripts/release/tests/release-governance-runner.test.mjs`
- Modify: `scripts/release-governance-workflow-contracts.mjs`
- Modify: `scripts/release-governance-workflow.test.mjs`

- [ ] **Step 1: Add a missing-plan assertion**

Require the release governance runner to expose a dedicated plan id for:

- `scripts/release/tests/materialize-external-deps.test.mjs`

- [ ] **Step 2: Require the workflow contract surface to cover that plan**

Keep the workflow assertion focused on the existing `run-release-governance-checks.mjs --profile preflight --format json` command while ensuring the runner contract test is the source of truth for the added preflight plan.

- [ ] **Step 3: Run the focused tests to verify RED**

Run:

```bash
node --test scripts/release-governance-workflow.test.mjs scripts/release/tests/release-governance-runner.test.mjs
```

Expected: FAIL because the preflight runner does not yet include `materialize-external-deps.test.mjs`.

### Task 2: Implement the missing preflight test plan

**Files:**
- Modify: `scripts/release/run-release-governance-checks.mjs`

- [ ] **Step 1: Add the preflight test plan entry**

Create a dedicated plan id for `scripts/release/tests/materialize-external-deps.test.mjs` and keep it inside the preflight profile.

- [ ] **Step 2: Add fallback handling if the runner supports in-process fallbacks for sibling contract tests**

Follow the existing fallback pattern only as far as needed for consistency.

- [ ] **Step 3: Re-run the focused tests to verify GREEN**

Run:

```bash
node --test scripts/release-governance-workflow.test.mjs scripts/release/tests/release-governance-runner.test.mjs
```

Expected: PASS.

### Task 3: Run focused regression verification

**Files:**
- Modify: none unless verification exposes a real defect

- [ ] **Step 1: Re-run the direct helper plus governance suite**

Run:

```bash
node --test scripts/release/tests/materialize-external-deps.test.mjs scripts/release-governance-workflow.test.mjs scripts/release/tests/release-governance-runner.test.mjs
```

Expected: PASS.

- [ ] **Step 2: Re-run the actual preflight command**

Run:

```bash
node scripts/release/run-release-governance-checks.mjs --profile preflight --format json
```

Expected: JSON summary with `"ok": true`.

- [ ] **Step 3: Inspect the targeted diff**

Run:

```bash
git diff -- scripts/release/run-release-governance-checks.mjs scripts/release/tests/release-governance-runner.test.mjs scripts/release-governance-workflow-contracts.mjs scripts/release-governance-workflow.test.mjs docs/superpowers/specs/2026-04-15-release-governance-materialize-external-deps-test-design.md docs/superpowers/plans/2026-04-15-release-governance-materialize-external-deps-test.md
```

Expected: only the release-governance materialize-external-deps test-execution slice appears.
