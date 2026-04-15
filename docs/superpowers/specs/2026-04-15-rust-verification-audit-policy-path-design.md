# Rust Verification Audit Policy Path Design

**Date:** 2026-04-15

## Goal

Ensure pull-request Rust verification is re-run whenever the dependency-audit policy file changes.

## Current Evidence

- `.github/workflows/rust-verification.yml` runs `node scripts/check-rust-verification-matrix.mjs --group dependency-audit`.
- `scripts/check-rust-verification-matrix.mjs` calls `scripts/check-rust-dependency-audit.mjs`.
- `scripts/check-rust-dependency-audit.mjs` loads `scripts/check-rust-dependency-audit.policy.json` as a required input.
- the current PR workflow path filter does not include `scripts/check-rust-dependency-audit.policy.json`.

## Problem Statement

The Rust verification workflow is not watching one of its core governance inputs. A PR that changes the dependency-audit allowlist policy can bypass the `rust-verification` workflow even though that policy directly changes audit behavior.

This is a commercial governance gap because:

- dependency-audit policy drift can merge without PR-time validation
- the approval surface for Rust security exceptions is not mechanically enforced
- release and product workflows may discover policy regressions later than necessary

## Recommendation

Add `scripts/check-rust-dependency-audit.policy.json` to `.github/workflows/rust-verification.yml` and assert it in `scripts/rust-verification-workflow.test.mjs`.

This is the smallest fix that aligns workflow triggers with actual dependency-audit behavior.

## Verification Boundary

The slice is acceptable only if:

- `node --test scripts/rust-verification-workflow.test.mjs` passes
- `node --test scripts/check-rust-dependency-audit.test.mjs scripts/check-rust-verification-matrix.test.mjs scripts/rust-verification-workflow.test.mjs` passes

## Success Condition

This work is successful when editing `scripts/check-rust-dependency-audit.policy.json` necessarily triggers PR-time Rust verification and the workflow test fails if that watched path disappears.
