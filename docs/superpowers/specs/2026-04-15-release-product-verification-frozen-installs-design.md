# Release Product Verification Frozen Installs Design

**Date:** 2026-04-15

## Goal

Close the release reproducibility gap where the `product-verification` workflow job can currently fall back to mutable `pnpm install` behavior inside `check-router-product.mjs` instead of using explicit, frozen dependency installation as part of the governed release pipeline.

The goal is not to change local developer ergonomics. The goal is to make the release gate deterministic and auditable.

## Current Evidence

- `.github/workflows/release.yml` now has a dedicated `product-verification` job and both `native-release` and `web-release` depend on it.
- `product-verification` runs `node scripts/check-router-product.mjs`.
- `scripts/check-router-product.mjs` calls `ensureFrontendAppReady(...)` for `apps/sdkwork-router-admin` and `apps/sdkwork-router-portal`.
- when installs are missing or unhealthy, `ensureFrontendAppReady(...)` executes `pnpm --dir <app> install` without `--frozen-lockfile`.
- the release workflow already uses explicit `pnpm install --frozen-lockfile` in `native-release` and `web-release`, but not in `product-verification`.

## Problem Statement

The release pipeline now enforces product verification, but the dependency preparation inside that gate is still implicit and mutable. That leaves a commercial-delivery risk:

- release verification can depend on an on-the-fly `pnpm install`
- lockfile drift or registry-side changes are not explicitly blocked at the start of the gate
- the workflow contract does not currently guarantee deterministic install behavior before `check-router-product.mjs` runs

This is a release-governance and reproducibility issue, not just a convenience detail.

## Options Considered

### Option A: Keep relying on `check-router-product.mjs` auto-install behavior

Pros:

- no workflow churn
- local and CI behavior stay identical

Cons:

- release verification remains dependent on mutable install behavior
- reproducibility is implicit rather than enforced
- workflow tests cannot guarantee frozen install discipline

### Option B: Add explicit frozen installs to `product-verification` and protect them with contract tests

Pros:

- makes the release workflow deterministic
- aligns `product-verification` with the explicit install discipline already used by `native-release` and `web-release`
- keeps `check-router-product.mjs` unchanged for local developer convenience
- small, focused governance change

Cons:

- adds one more workflow step to maintain
- still leaves the script capable of mutable installs outside release CI

### Option C: Make `check-router-product.mjs` hard-fail in release CI when installs are missing

Pros:

- strongest enforcement at the script boundary
- prevents accidental fallback even if workflow wiring regresses

Cons:

- broader behavior change
- requires new script-level interface or CI mode handling
- larger slice than the actual current release defect

## Recommendation

Choose Option B.

It closes the real release-chain defect with the smallest safe change: the workflow becomes explicit about dependency installation, while local `product:check` behavior stays ergonomic. Option C can follow later if the team wants stricter CI-only behavior at the script layer.

## Design

### Workflow Boundary

Extend the `product-verification` job in `.github/workflows/release.yml` with a dedicated install step before `Run release product verification`:

```bash
pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
pnpm --dir apps/sdkwork-router-portal install --frozen-lockfile
```

This step should run after Node and pnpm setup, and before `node scripts/check-router-product.mjs`.

### Scope Boundary

Only install the workspaces that `check-router-product.mjs` and `build-router-desktop-assets.mjs` actually need:

- `apps/sdkwork-router-admin`
- `apps/sdkwork-router-portal`

Do not widen the step to `console` or `docs` in this slice because they are not part of the product-verification script path.

### Contract Boundary

Extend:

- `scripts/release/release-workflow-contracts.mjs`
- `scripts/release/tests/release-workflow.test.mjs`

to require that:

- `product-verification` includes an explicit frozen install step
- that step happens before `Run release product verification`
- the step installs both admin and portal workspaces with `--frozen-lockfile`
- the contract helper rejects workflows where the step is missing

### Verification Boundary

The slice is acceptable only if all of the following pass:

- `node --test scripts/release/tests/release-workflow.test.mjs`
- `node --test scripts/check-router-product.test.mjs apps/sdkwork-router-portal/tests/product-entrypoint-scripts.test.mjs`
- `node --test scripts/check-router-docs-safety.test.mjs`

## Risks And Mitigations

### Risk: workflow stays green while verification still performs mutable installs

Mitigation:

- assert the explicit frozen install step in both direct workflow tests and the contract helper

### Risk: change accidentally widens install scope and slows release verification unnecessarily

Mitigation:

- restrict the install step to the exact app directories used by `check-router-product.mjs`

### Risk: local developer flow becomes more rigid than necessary

Mitigation:

- keep `check-router-product.mjs` behavior unchanged in this slice
- enforce frozen installs at the release workflow layer only

## Success Condition

This work is successful when `product-verification` no longer relies on implicit mutable installs during release CI:

- the workflow explicitly installs admin and portal dependencies with `--frozen-lockfile`
- release workflow tests fail if that step is removed
- docs safety and product verification remain covered by the existing release gate without adding redundant jobs
