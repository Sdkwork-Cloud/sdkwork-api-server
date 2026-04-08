# Unreleased - Step 10 Release Governance Latest Artifact Restore

- Date: 2026-04-08
- Type: patch
- Highlights:
  - added `scripts/release/restore-release-governance-latest.mjs`, giving blocked hosts a repository-owned command that restores downloaded governance artifacts into the default latest paths under `docs/release/`
  - enforced validator-backed restore rules for release-window, release-sync, telemetry export, telemetry snapshot, and SLO evidence, including hard rejection of conflicting duplicate artifacts
  - added end-to-end proof that after restoring real latest artifacts, `runReleaseGovernanceChecks()` can replay cleanly even when Node child execution is forced to `EPERM`
  - kept release truth honest: restore rehydrates governed evidence only; it does not synthesize fresh Git or telemetry facts
