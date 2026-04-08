# 2026-04-08 Release Governance Latest Artifact Restore Review

## 1. Scope

- `scripts/release/restore-release-governance-latest.mjs`
- `scripts/release/tests/restore-release-governance-latest.test.mjs`

## 2. Finding

### P1 blocked-host replay still lacked an operator handoff path even after latest artifacts were standardized

- The repo already owned:
  - latest artifact producers
  - workflow upload
  - attestation coverage
  - governance replay from default latest paths
- It still did not own:
  - a repository script that restores downloaded governance artifacts back into the default latest paths on restricted hosts

## 3. Root Cause

- Governance replay assumed the latest files already existed at:
  - `docs/release/release-window-snapshot-latest.json`
  - `docs/release/release-sync-audit-latest.json`
  - `docs/release/release-telemetry-export-latest.json`
  - `docs/release/release-telemetry-snapshot-latest.json`
  - `docs/release/slo-governance-latest.json`
- Workflow could produce these files, but local operators had no repository-owned restore command to hydrate them from downloaded artifacts.

## 4. Changes

- Added `scripts/release/restore-release-governance-latest.mjs`
  - restores all required latest governance artifacts from a downloaded artifact root or explicit file paths
  - validates each restored artifact with the repository’s existing contracts before writing
  - allows duplicate artifacts when their payloads are canonically identical
  - rejects conflicting duplicates instead of silently choosing one
- Added `scripts/release/tests/restore-release-governance-latest.test.mjs`
  - restore from downloaded artifact directory
  - identical duplicate tolerance
  - conflicting duplicate rejection
  - end-to-end blocked-host governance replay after restoration

## 5. Verification

- `restore-release-governance-latest.test.mjs`
  - `4 / 4`
- blocked-host replay proof inside the test now shows:
  - restore latest artifacts
  - force Node child execution to `EPERM`
  - `runReleaseGovernanceChecks()` returns all lanes passing from restored latest artifacts

## 6. Current Truth

- The repository now owns both halves of the governed replay path:
  - producer side in workflow
  - restore side on blocked hosts
- Default local runs are still honestly blocked until real downloaded artifacts are restored.
- No synthetic release truth was committed to the repository.

## 7. Next Step

1. If operator ergonomics still matter, add a dedicated bundled governance download artifact or manifest to reduce multi-artifact download friction.
2. Keep `release-slo-governance` blocked by default until real telemetry evidence is restored or materialized.
