# Unreleased - Step 10 Release Governance Default Latest CLI Replay

- Date: 2026-04-08
- Type: patch
- Highlights:
  - updated `compute-release-window-snapshot.mjs` and `verify-release-sync.mjs` so restored default latest artifacts are now consumed by the real CLI path before any live Git attempt
  - added regression coverage proving both lanes skip live Git when `docs/release/release-window-snapshot-latest.json` or `docs/release/release-sync-audit-latest.json` already exists
  - manually verified the operator flow on this host: restore `5` governed artifacts, then run `run-release-governance-checks.mjs --format json`, which now returns `ok=true` and no blocked/failing lanes
  - kept release truth honest: the repository still blocks by default when latest artifacts or telemetry evidence are absent
