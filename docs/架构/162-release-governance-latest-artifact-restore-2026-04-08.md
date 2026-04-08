# Release Governance Latest Artifact Restore

> Date: 2026-04-08
> Goal: give blocked hosts a repository-owned way to restore downloaded governance evidence back into the standard latest artifact paths.

## 1. Problem

- The workflow can already produce and upload governed latest artifacts.
- The governance runner can already replay those artifacts when they exist locally.
- Blocked hosts still lacked a repository command that turns downloaded workflow artifacts into those local latest files.

## 2. Restore Contract

- Entry:
  - `scripts/release/restore-release-governance-latest.mjs`
- Sources:
  - `--artifact-dir <dir>`
  - or explicit per-artifact paths:
    - `--window`
    - `--sync`
    - `--telemetry-export`
    - `--telemetry-snapshot`
    - `--slo`
- Output root:
  - default repo root
  - optional `--repo-root <dir>` for testing or staging

## 3. Required Restored Files

- `docs/release/release-window-snapshot-latest.json`
- `docs/release/release-sync-audit-latest.json`
- `docs/release/release-telemetry-export-latest.json`
- `docs/release/release-telemetry-snapshot-latest.json`
- `docs/release/slo-governance-latest.json`

## 4. Validation Rules

- Every restored file must pass the repository’s existing validators:
  - release-window artifact validator
  - release-sync artifact validator
  - telemetry export validator
  - telemetry snapshot validator
  - SLO evidence validator
- Duplicate artifacts are allowed only when their canonical JSON payload is identical.
- Conflicting duplicates are a hard error.

## 5. Replay Effect

- After restore, blocked-host replay works through the existing default lookup order:
  - `release-window-snapshot`
  - `release-sync-audit`
  - `release-slo-governance` via restored telemetry export / snapshot / SLO evidence
- This does not change live-host truth.
- It only hydrates already-produced governed evidence into the paths that the repo already recognizes.

## 6. Honest Boundary

- Restore does not fabricate evidence.
- Restore does not recompute Git or telemetry facts.
- Restore only rehydrates previously produced governed artifacts.

## 7. Remaining Closure

- Operators still need a download source for these artifacts.
- If multi-artifact download remains cumbersome, the next step is a bundled governance evidence artifact or manifest.
