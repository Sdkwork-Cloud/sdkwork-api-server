# Rust Verification Helper Input Paths Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** ensure the `rust-verification` pull-request workflow runs when shared helper modules behind `check-rust-verification-matrix.mjs` change.

**Architecture:** keep the existing Rust verification matrix behavior unchanged, then harden only the PR trigger surface so the workflow watches the direct and transitive helper inputs that shape the matrix execution environment.

**Tech Stack:** GitHub Actions workflow YAML, Node test runner, Rust verification support scripts.

---

## File Map

- Create: `docs/superpowers/specs/2026-04-15-rust-verification-helper-input-paths-design.md`
  - capture the missing helper-input trigger coverage and the recommended minimal fix.
- Modify: `scripts/rust-verification-workflow.test.mjs`
  - assert that the workflow watches the shared helper inputs.
- Modify: `.github/workflows/rust-verification.yml`
  - add the helper modules to `pull_request.paths`.

### Task 1: Write the failing workflow assertions first

**Files:**
- Modify: `scripts/rust-verification-workflow.test.mjs`

- [ ] **Step 1: Add the helper-path assertions**

Require the workflow to watch:

- `scripts/run-tauri-cli.mjs`
- `scripts/workspace-target-dir.mjs`
- `scripts/release/desktop-targets.mjs`

- [ ] **Step 2: Run the workflow test to verify RED**

Run:

```bash
node --test scripts/rust-verification-workflow.test.mjs
```

Expected: FAIL because the workflow does not yet watch the helper inputs.

### Task 2: Implement the workflow trigger fix

**Files:**
- Modify: `.github/workflows/rust-verification.yml`

- [ ] **Step 1: Expand the path filter**

Add:

```yaml
- 'scripts/run-tauri-cli.mjs'
- 'scripts/workspace-target-dir.mjs'
- 'scripts/release/desktop-targets.mjs'
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
git diff -- .github/workflows/rust-verification.yml scripts/rust-verification-workflow.test.mjs docs/superpowers/specs/2026-04-15-rust-verification-helper-input-paths-design.md docs/superpowers/plans/2026-04-15-rust-verification-helper-input-paths.md
```

Expected: only the rust-verification helper-input hardening slice appears.
