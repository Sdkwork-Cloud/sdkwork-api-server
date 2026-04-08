# Unreleased - Step 10 Release Window / Sync Governed Latest Artifacts

- Date: 2026-04-08
- Type: patch
- Highlights:
  - added `materialize-release-window-snapshot.mjs` and `materialize-release-sync-audit.mjs`, so both Git-based governance lanes now have repository-owned latest artifact producers
  - updated `.github/workflows/release.yml` so native and web release jobs materialize, upload, and attest `release-window-snapshot-latest.json` and `release-sync-audit-latest.json`
  - updated `run-release-governance-checks.mjs` so blocked-host replay now prefers explicit governed env input, then repository-owned default latest artifacts, before falling back to live Git
  - extended `verify-release-attestations.mjs` and release workflow / attestation contracts so both new artifacts are part of executable release truth
  - re-verified the new producer tests plus the updated release governance, workflow, and attestation suites
  - kept release truth honest: local default runs still block until latest artifacts or telemetry evidence are actually supplied
