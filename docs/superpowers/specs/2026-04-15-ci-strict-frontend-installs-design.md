# CI Strict Frontend Installs Design

**Date:** 2026-04-15

## Goal

Eliminate the remaining release-governance gap where frontend verification scripts can still fall back to implicit `pnpm install` repair behavior during CI or release execution.

The objective is release determinism:

- keep local developer workflows ergonomic
- make CI and release lanes fail fast when frontend dependencies are missing or unhealthy
- avoid hidden dependency mutations after the workflow has already declared its install policy

## Current Evidence

- `check-router-product.mjs` uses `ensureFrontendAppReady(...)`, which runs `pnpm install` when frontend dependencies are missing or unhealthy.
- `build-router-desktop-assets.mjs` duplicates the same repair logic and also runs `pnpm install`.
- `.github/workflows/release.yml` now explicitly installs portal/admin dependencies with `--frozen-lockfile` before `product-verification`, but the script can still repair installs if the workspace is later judged unhealthy.
- both scripts use the same readiness primitives from `scripts/dev/pnpm-launch-lib.mjs`, but the strictness policy is not centralized.

## Problem Statement

The release workflow now declares a frozen-install policy, but the script layer still allows mutation afterward. This is a commercial-readiness problem because:

- the workflow says installs are governed, but scripts can still mutate dependencies
- missing or unhealthy installs become hidden repairs instead of explicit release failures
- duplicate repair logic in multiple scripts makes future governance drift more likely

## Options Considered

### Option A: Rely only on workflow-level frozen installs

Pros:

- smallest surface area
- no script changes required

Cons:

- script-level fallback still exists
- future workflow regressions can silently reactivate mutable repair behavior
- duplicated repair logic remains ungoverned

### Option B: Add CI strict mode only to `check-router-product.mjs`

Pros:

- fixes the current product verification path
- small script change

Cons:

- `build-router-desktop-assets.mjs` still keeps equivalent mutable behavior
- strictness policy remains duplicated

### Option C: Centralize strict frontend install policy in `pnpm-launch-lib.mjs`

Pros:

- one source of truth for frontend dependency readiness policy
- both product verification and desktop asset build paths stay aligned
- release workflow can opt into strict mode without harming local dev ergonomics

Cons:

- slightly broader than a single-script patch
- requires tests in both shared helper and workflow/script call sites

## Recommendation

Choose Option C.

This keeps the scope focused on frontend dependency governance while removing duplicated policy logic. It is the most robust path for long-term commercial delivery because CI expectations become explicit and shared.

## Design

### Shared Policy Boundary

Add shared strict-mode helpers to `scripts/dev/pnpm-launch-lib.mjs`:

- parse whether strict frontend install mode is enabled from environment
- when strict mode is enabled and frontend dependencies are not `ready`, throw instead of running `pnpm install`
- when strict mode is disabled, preserve the current repair-in-place behavior

The release-safe error should clearly explain that a prior frozen install step is required.

### Script Boundary

Update both scripts to use the shared helper:

- `scripts/check-router-product.mjs`
- `scripts/build-router-desktop-assets.mjs`

They should no longer own separate policy decisions about strict vs repair mode.

### Workflow Boundary

Set strict mode explicitly in the `product-verification` release job so that:

- release verification fails if dependencies are not ready after the frozen install step
- the workflow cannot silently mutate dependencies during verification

This can be achieved with an environment variable such as `SDKWORK_STRICT_FRONTEND_INSTALLS: '1'`.

### Test Boundary

Add red-green coverage for:

- strict-mode detection in `pnpm-launch-lib`
- strict mode refusing non-ready frontend installs
- workflow wiring that exports the strict mode env into `product-verification`

## Risks And Mitigations

### Risk: local developer ergonomics regress

Mitigation:

- strict mode is opt-in via environment variable
- default local behavior remains repair-in-place

### Risk: only one script receives the policy and the other drifts later

Mitigation:

- centralize strict-mode behavior in `pnpm-launch-lib.mjs`
- update both scripts to consume the same helper

### Risk: workflow claims strictness but does not export the env

Mitigation:

- add release workflow contract assertions for the strict mode env wiring

## Success Condition

This work is successful when:

- CI and release verification can no longer repair frontend installs implicitly
- local scripts still retain repair behavior by default
- both `check-router-product.mjs` and `build-router-desktop-assets.mjs` consume the same strictness policy
- workflow tests fail if `product-verification` stops exporting strict install mode
