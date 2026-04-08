# Release Window / Sync Governed Latest Artifacts

> Date: 2026-04-08
> Goal: promote `release-window-snapshot` and `release-sync-audit` from governed-input-only lanes into repository-owned latest artifacts that workflow, attestation, and blocked-host replay can consume consistently.

## 1. Problem

- The repo already had:
  - live Git computation
  - governed input validation
- It still lacked:
  - producer scripts for standard latest artifacts
  - workflow upload
  - attestation subject coverage
  - default replay of repository-owned latest artifacts

## 2. Artifact Contract

- Release window latest artifact:
  - `docs/release/release-window-snapshot-latest.json`
  - envelope:
    - `version`
    - `generatedAt`
    - `source`
    - `snapshot`
- Release sync latest artifact:
  - `docs/release/release-sync-audit-latest.json`
  - envelope:
    - `version`
    - `generatedAt`
    - `source`
    - `summary`

## 3. Producer Contract

- `scripts/release/materialize-release-window-snapshot.mjs`
  - direct governed input:
    - `--snapshot`
    - `--snapshot-json`
    - `SDKWORK_RELEASE_WINDOW_SNAPSHOT_PATH`
    - `SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON`
  - live mode:
    - derives snapshot from Git
- `scripts/release/materialize-release-sync-audit.mjs`
  - direct governed input:
    - `--audit`
    - `--audit-json`
    - `SDKWORK_RELEASE_SYNC_AUDIT_PATH`
    - `SDKWORK_RELEASE_SYNC_AUDIT_JSON`
  - live mode:
    - derives multi-repository sync facts from Git

## 4. Workflow Contract

1. `Materialize external release dependencies`
2. `Materialize release window snapshot`
3. `Upload release window snapshot governance artifact`
4. `Materialize release sync audit`
5. `Upload release sync audit governance artifact`
6. `Materialize release telemetry export`
7. `Upload release telemetry export governance artifact`
8. `Materialize release telemetry snapshot`
9. `Upload release telemetry snapshot governance artifact`
10. `Materialize SLO governance evidence`
11. `Upload SLO governance evidence artifact`
12. `Generate governance evidence attestation`
13. `Run release governance gate`

- Governance gate input now explicitly includes:
  - `SDKWORK_RELEASE_WINDOW_SNAPSHOT_PATH=docs/release/release-window-snapshot-latest.json`
  - `SDKWORK_RELEASE_SYNC_AUDIT_PATH=docs/release/release-sync-audit-latest.json`

## 5. Replay Contract

- `run-release-governance-checks.mjs` fallback order for these two lanes:
  1. explicit governed env input
  2. repository-owned default latest artifact
  3. live Git replay
  4. blocked result if live Git is denied

## 6. Attestation Contract

- Required governed evidence subjects now include:
  - `release-window-snapshot`
  - `release-sync-audit`
  - `release-telemetry-export`
  - `release-telemetry-snapshot`
  - `release-slo-governance`

## 7. Honest Boundary

- This slice does not claim local hosts now have live Git capability.
- It does not commit synthetic latest artifacts as repository truth.
- It only ensures the repository can:
  - produce standard latest artifacts on allowed hosts
  - attest them in workflow
  - replay them on blocked hosts when explicitly supplied

## 8. Remaining Closure

- Default local replay still blocks until latest artifacts exist.
- `release-slo-governance` still depends on upstream telemetry export availability.
- Operator retention and artifact pullback policy is still a separate control-plane concern.
