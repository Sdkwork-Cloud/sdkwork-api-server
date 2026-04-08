# Unreleased - Step 10 Release Governance Bundle

- Date: 2026-04-09
- Type: patch
- Highlights:
  - added `scripts/release/materialize-release-governance-bundle.mjs`, which validates and packages the five governed latest artifacts into one restore-oriented bundle plus manifest
  - updated `.github/workflows/release.yml` so `web-release` now uploads `release-governance-bundle-web`
  - updated workflow contracts and tests so the single-download operator handoff is now repository-enforced
  - re-ran the full `scripts/release/tests/*.test.mjs` suite at `76 / 76`
  - kept release truth honest: `run-release-governance-checks.mjs --format json` on this host still reports `7` pass, `3` block, `0` fail until real live inputs or restored evidence are present
