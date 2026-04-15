# Rust Verification Audit Policy Path Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** ensure the `rust-verification` pull-request workflow runs when the dependency-audit policy file changes.

**Architecture:** keep the existing Rust verification matrix intact, then expand the workflow path filter and direct workflow test so the dependency-audit policy file is treated as a first-class governance input.

**Tech Stack:** GitHub Actions workflow YAML, Node test runner, Rust governance scripts.

---

## File Map

- Create: `docs/superpowers/specs/2026-04-15-rust-verification-audit-policy-path-design.md`
  - capture the missing policy-input trigger and recommended fix.
- Modify: `scripts/rust-verification-workflow.test.mjs`
  - assert the policy file is watched.
- Modify: `.github/workflows/rust-verification.yml`
  - add the policy file to `pull_request.paths`.

### Task 1: Write the failing workflow test first

**Files:**
- Modify: `scripts/rust-verification-workflow.test.mjs`

- [ ] **Step 1: Add the policy-path assertion**

Require the workflow to contain:

- `scripts/check-rust-dependency-audit.policy.json`

- [ ] **Step 2: Run the test to verify RED**

Run:

```bash
node --test scripts/rust-verification-workflow.test.mjs
```

Expected: FAIL because the workflow does not yet watch the policy file.

### Task 2: Implement the workflow trigger fix

**Files:**
- Modify: `.github/workflows/rust-verification.yml`

- [ ] **Step 1: Expand the path filter**

Add:

```yaml
- 'scripts/check-rust-dependency-audit.policy.json'
```

to the workflow trigger paths.

- [ ] **Step 2: Re-run the workflow test to verify GREEN**

Run:

```bash
node --test scripts/rust-verification-workflow.test.mjs
```

Expected: PASS.

### Task 3: Run focused regression verification

**Files:**
- Modify: none unless verification exposes a real regression

- [ ] **Step 1: Re-run Rust governance support tests**

Run:

```bash
node --test scripts/check-rust-dependency-audit.test.mjs scripts/check-rust-verification-matrix.test.mjs scripts/rust-verification-workflow.test.mjs
```

Expected: PASS.

- [ ] **Step 2: Inspect the targeted diff**

Run:

```bash
git diff -- .github/workflows/rust-verification.yml scripts/rust-verification-workflow.test.mjs docs/superpowers/specs/2026-04-15-rust-verification-audit-policy-path-design.md docs/superpowers/plans/2026-04-15-rust-verification-audit-policy-path.md
```

Expected: only the rust-verification policy-path hardening slice appears.
