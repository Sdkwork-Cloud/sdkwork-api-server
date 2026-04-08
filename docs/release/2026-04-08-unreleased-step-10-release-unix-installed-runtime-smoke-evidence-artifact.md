# Unreleased - Step 10 Release Unix Installed Runtime Smoke Evidence Artifact

- Date: 2026-04-08
- Type: patch
- Summary:
  - extended `scripts/release/run-unix-installed-runtime-smoke.mjs` with `--evidence-path` and JSON evidence writing for both success and failure paths
  - inserted a dedicated Unix smoke evidence upload step into `.github/workflows/release.yml` using `release-governance-unix-installed-runtime-smoke-*`
  - kept governance evidence out of `release-assets-*`, so publish artifacts remain customer-facing only
  - hardened workflow contracts and tests so removing the evidence path or upload step now fails verification immediately
- Verification:
  - `run-unix-installed-runtime-smoke.test.mjs`: `2 / 2`
  - `release-workflow.test.mjs`: `9 / 9`
  - `release-governance-runner.test.mjs`: `9 / 9`
  - governance summary: `6` pass / `3` block / `0` fail
- Remaining truth:
  - this session verified contract wiring and evidence persistence, not a local full built-artifact Unix release smoke
  - Windows installed-runtime evidence remains follow-up work
  - live SLO/Git-based release lanes remain blocked for the same reasons as before
