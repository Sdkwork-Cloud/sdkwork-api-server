# Unreleased - Step 10 Release Unix Installed Runtime Smoke Gate

- Date: 2026-04-08
- Type: patch
- Summary:
  - added `scripts/release/run-unix-installed-runtime-smoke.mjs` so native release lanes can install real built outputs into a temporary runtime home, execute installed `start.sh`, probe unified health endpoints, and execute installed `stop.sh`
  - inserted `Run installed native runtime smoke on Unix` into `.github/workflows/release.yml` after native builds and before native asset packaging
  - hardened `release-workflow` contracts and tests so removal or reordering of the Unix installed-runtime gate now breaks verification immediately
- Verification:
  - `release-workflow.test.mjs`: `8 / 8`
  - `run-unix-installed-runtime-smoke.test.mjs`: `2 / 2`
  - `release-governance-runner.test.mjs`: `9 / 9`
  - governance summary: `6` pass / `3` block / `0` fail
- Remaining truth:
  - local session did not run a full built-artifact Unix smoke because release binaries were not built here
  - Windows installed-runtime gating remains follow-up work
  - live Git/evidence release lanes remain host-blocked, not regressed
