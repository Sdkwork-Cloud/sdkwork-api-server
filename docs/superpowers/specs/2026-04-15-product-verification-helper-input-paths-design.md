# Product Verification Helper Input Paths Design

**Date:** 2026-04-15

## Goal

Ensure the pull-request `product-verification` workflow re-runs when shared desktop/runtime helper scripts used by the product gate change.

## Current Evidence

- `scripts/check-router-product.mjs` imports `withSupportedWindowsCmakeGenerator` from `scripts/run-tauri-cli.mjs`.
- `scripts/run-router-product-service.mjs` also imports `withSupportedWindowsCmakeGenerator` from `scripts/run-tauri-cli.mjs`.
- `scripts/run-tauri-cli.mjs` imports `buildDesktopReleaseEnv` from `scripts/release/desktop-targets.mjs`.
- `.github/workflows/product-verification.yml` does not currently watch either helper file.

## Problem Statement

The product verification gate depends on shared helper code that can change product build and runtime planning behavior, but PR-time path filtering does not currently observe those files.

That leaves a governance gap:

- helper regressions can merge without re-running `product-verification`
- import-time breakage in shared helper modules can bypass the product gate
- desktop/runtime helper drift is treated as unrelated even though product verification depends on it

## Recommendation

Add both helper files to `product-verification.yml` and enforce them in workflow tests and contract assertions:

- `scripts/run-tauri-cli.mjs`
- `scripts/release/desktop-targets.mjs`

## Verification Boundary

The slice is acceptable only if:

- `node --test scripts/product-verification-workflow.test.mjs` passes
- `node --test scripts/check-router-product.test.mjs scripts/build-router-desktop-assets.test.mjs scripts/check-router-docs-safety.test.mjs scripts/check-router-frontend-budgets.test.mjs apps/sdkwork-router-portal/tests/product-entrypoint-scripts.test.mjs` passes

## Success Condition

This work is successful when changing either shared helper file necessarily re-runs the pull-request product verification workflow and the workflow contract tests fail if that protection is removed.
