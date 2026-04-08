# Release Governance Telemetry Snapshot Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a repository-owned release telemetry snapshot contract so the release workflow materializes a governed snapshot first and derives SLO evidence from that snapshot instead of relying on a raw env JSON payload.

**Architecture:** Introduce a dedicated snapshot materializer under `scripts/release`, keep SLO evidence materialization as a second governed step, and lock the workflow ordering with tests. The workflow remains honest: no fake committed telemetry artifact is added; the live SLO lane stays blocked until real telemetry is supplied.

**Tech Stack:** Node.js ESM scripts, GitHub Actions workflow contracts, `node:test`, release docs under `docs/review`, `docs/架构`, `docs/release`, and `docs/step`.

---

### Task 1: Lock the new snapshot contract in tests

**Files:**
- Create: `scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs`
- Modify: `scripts/release/tests/materialize-slo-governance-evidence.test.mjs`
- Modify: `scripts/release/tests/release-workflow.test.mjs`
- Modify: `scripts/release/release-workflow-contracts.mjs`

- [ ] **Step 1: Write failing tests for snapshot helper exports and artifact shape**
- [ ] **Step 2: Run the new snapshot test and confirm it fails for missing helper/script**
- [ ] **Step 3: Extend SLO materializer tests so snapshot-derived evidence is required**
- [ ] **Step 4: Extend workflow tests/contracts so release jobs must materialize telemetry snapshot before SLO evidence**

### Task 2: Implement the telemetry snapshot producer and SLO derivation

**Files:**
- Create: `scripts/release/materialize-release-telemetry-snapshot.mjs`
- Modify: `scripts/release/materialize-slo-governance-evidence.mjs`

- [ ] **Step 1: Add snapshot input resolution and shape validation**
- [ ] **Step 2: Write the governed snapshot artifact to `docs/release/release-telemetry-snapshot-latest.json`**
- [ ] **Step 3: Let SLO evidence materialization derive governed evidence from snapshot input while keeping direct evidence support for local/manual use**
- [ ] **Step 4: Re-run targeted tests until green**

### Task 3: Wire the release workflow to the new truth source

**Files:**
- Modify: `.github/workflows/release.yml`

- [ ] **Step 1: Add `Materialize release telemetry snapshot` before `Materialize SLO governance evidence` in native and web jobs**
- [ ] **Step 2: Change SLO materialization to consume the governed snapshot artifact path instead of raw evidence JSON env**
- [ ] **Step 3: Re-run workflow tests/contracts**

### Task 4: Record the review and the remaining live blockers

**Files:**
- Create: `docs/review/2026-04-08-release-governance-telemetry-snapshot-contract-review.md`
- Create: `docs/架构/148-release-telemetry-snapshot-governance-2026-04-08.md`
- Create: `docs/release/2026-04-08-unreleased-step-10-release-governance-telemetry-snapshot-contract.md`
- Create: `docs/step/2026-04-08-release-governance-telemetry-snapshot-contract-step-update.md`
- Modify: `docs/release/CHANGELOG.md`

- [ ] **Step 1: Record the bug, root cause, chosen design, and exact scope**
- [ ] **Step 2: Record verification output and unchanged blockers**
- [ ] **Step 3: Set the next slice to real telemetry export ownership/freshness**
