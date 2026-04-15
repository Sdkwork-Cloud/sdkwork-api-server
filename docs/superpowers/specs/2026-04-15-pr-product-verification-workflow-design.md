# PR Product Verification Workflow Design

**Date:** 2026-04-15

## Goal

Move product-grade verification from release-only enforcement to pull-request-time enforcement so commercial regressions are detected before a release tag is cut.

This slice is focused on feedback timing, not on inventing new checks. The existing product verification logic should stay the source of truth.

## Current Evidence

- `.github/workflows/release.yml` now contains a dedicated `product-verification` job.
- that release job executes `node scripts/check-router-product.mjs` after explicit frozen installs and with strict frontend install mode enabled.
- the repository currently exposes PR-time workflow coverage only for `rust-verification.yml`.
- there is no tracked PR workflow that runs `check-router-product.mjs`, docs safety, desktop asset budget checks, or the product service deployment plan before release.

## Problem Statement

The product verification gate exists, but it mainly runs during release preparation. That leaves a commercial-delivery gap:

- product regressions can survive until release time
- release readiness is evaluated too late in the change lifecycle
- the team pays the highest debugging cost at the most expensive point in the delivery flow

For commercial operation, the issue is not missing logic. The issue is that the logic is enforced too late.

## Options Considered

### Option A: Extend `rust-verification.yml` to also run product verification

Pros:

- fewer workflow files
- existing PR workflow already exists

Cons:

- mixes Rust package-group verification with full product verification semantics
- path filters and dispatch model become harder to reason about
- obscures ownership: product governance is not the same concern as Rust verification

### Option B: Add a dedicated `product-verification.yml` PR workflow

Pros:

- clear responsibility boundary
- mirrors the release `product-verification` gate explicitly
- easier contract testing and path filtering
- simpler for operators to understand in CI dashboards

Cons:

- adds another workflow file to maintain
- requires dedicated contract tests

### Option C: Depend on release workflow only

Pros:

- no additional workflow work

Cons:

- regressions are caught too late
- weak commercial feedback loop
- product gate stays release-centric instead of change-centric

## Recommendation

Choose Option B.

This keeps product verification as a first-class CI concern with a clean boundary, while reusing the existing `check-router-product.mjs` logic. It is the most maintainable and commercially defensible option.

## Design

### Workflow Boundary

Create `.github/workflows/product-verification.yml` with:

- `pull_request` trigger and path filters for product-facing apps, docs, Rust service/runtime code, and the scripts/workflow files that influence product verification
- `workflow_dispatch` trigger for manual replay
- a single `product-verification` job on `ubuntu-latest`

### Job Boundary

The job should mirror the release gate's dependency preparation discipline:

- checkout repository
- setup pnpm
- setup Node.js 22 with pnpm cache
- install Rust toolchain
- cache Rust dependencies
- install `cargo-audit`
- install admin and portal workspaces with `--frozen-lockfile`
- run lightweight governance Node tests for the workflow and product verification scripts
- run `node scripts/check-router-product.mjs` with `SDKWORK_STRICT_FRONTEND_INSTALLS: '1'`

### Contract Boundary

Add dedicated workflow contract coverage:

- `scripts/product-verification-workflow-contracts.mjs`
- `scripts/product-verification-workflow.test.mjs`

The contract should assert:

- the workflow exists and is PR-triggered
- it includes explicit frozen installs
- it exports strict frontend install mode
- it installs `cargo-audit`
- it executes `check-router-product.mjs`

### Verification Boundary

The slice is acceptable only if:

- `node --test scripts/product-verification-workflow.test.mjs` passes
- `node --test scripts/check-router-product.test.mjs scripts/build-router-desktop-assets.test.mjs scripts/check-router-docs-safety.test.mjs scripts/check-router-frontend-budgets.test.mjs apps/sdkwork-router-portal/tests/product-entrypoint-scripts.test.mjs` passes

## Risks And Mitigations

### Risk: PR workflow drifts away from release workflow semantics

Mitigation:

- explicitly mirror the release gate's frozen installs and strict frontend install env
- protect the workflow with a dedicated contract helper

### Risk: path filters miss product-facing regressions

Mitigation:

- include product apps, docs, Rust workspace/service files, scripts, and workflow files touched by `check-router-product.mjs`
- prefer slightly broader coverage over false negatives

### Risk: product workflow becomes a second source of truth

Mitigation:

- keep `check-router-product.mjs` as the only behavioral source of truth
- make the workflow orchestration thin and declarative

## Success Condition

This work is successful when product-grade regressions are blocked at PR time:

- a dedicated PR workflow runs the same governed product verification logic used by release
- frozen installs and strict frontend install mode are enforced there too
- contract tests fail if the PR workflow stops mirroring the release gate's critical safety properties
