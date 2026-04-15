# Rust Verification Helper Input Paths Design

**Date:** 2026-04-15

## Goal

Ensure pull-request Rust verification re-runs whenever shared helper modules that shape the verification matrix environment change.

## Current Evidence

- `.github/workflows/rust-verification.yml` runs `node scripts/check-rust-verification-matrix.mjs --group ${{ matrix.group }}`.
- `scripts/check-rust-verification-matrix.mjs` imports `withSupportedWindowsCmakeGenerator` from `scripts/run-tauri-cli.mjs`.
- `scripts/check-rust-verification-matrix.mjs` imports `withManagedWorkspaceTargetDir` from `scripts/workspace-target-dir.mjs`.
- `scripts/run-tauri-cli.mjs` imports `buildDesktopReleaseEnv` from `scripts/release/desktop-targets.mjs`.
- the current `pull_request.paths` filter in `rust-verification.yml` does not watch any of those three shared helper modules.

## Problem Statement

The Rust verification workflow is still vulnerable to a PR-time bypass through shared helper inputs.

A change to any of these helpers can alter or break the verification runtime without triggering the `rust-verification` workflow:

- Windows CMake generator selection in `scripts/run-tauri-cli.mjs`
- managed workspace target directory behavior in `scripts/workspace-target-dir.mjs`
- transitive desktop target environment loading in `scripts/release/desktop-targets.mjs`

This is a commercial-readiness gap because CI governance is only trustworthy when workflow triggers cover the real executable inputs, not just the top-level entry script.

## Options Considered

### Option A: Keep watching only the top-level matrix script

Pros:

- no workflow growth

Cons:

- leaves the current bypass intact
- does not reflect the actual helper import chain

### Option B: Watch only the direct helper imports

Pros:

- closes the obvious first-order gap
- small workflow diff

Cons:

- still leaves `scripts/release/desktop-targets.mjs` outside PR coverage even though `scripts/run-tauri-cli.mjs` imports it at module load

### Option C: Watch the full local helper chain used by the matrix script

Pros:

- aligns workflow triggers with the real runtime dependency path
- matches the already-established `product-verification` hardening pattern
- remains narrowly scoped to helper inputs that can directly influence the matrix runner

Cons:

- adds three watched paths instead of two

## Recommendation

Choose Option C.

The smallest trustworthy fix is to watch every local helper module that can change `check-rust-verification-matrix.mjs` behavior through direct import or transitive module loading:

- `scripts/run-tauri-cli.mjs`
- `scripts/workspace-target-dir.mjs`
- `scripts/release/desktop-targets.mjs`

## Verification Boundary

This slice is acceptable only if:

- `node --test scripts/rust-verification-workflow.test.mjs` passes
- `node --test scripts/check-rust-dependency-audit.test.mjs scripts/check-rust-verification-matrix.test.mjs scripts/rust-verification-workflow.test.mjs` passes

## Success Condition

This work is successful when editing any Rust verification helper input necessarily triggers the PR-time `rust-verification` workflow and the workflow regression test fails if any of those watched helper paths disappear.
