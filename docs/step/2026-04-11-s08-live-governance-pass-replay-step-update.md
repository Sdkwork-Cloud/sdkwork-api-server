# 2026-04-11 S08 Live Governance Pass Replay Step Update

## Scope

- Step: `S08 integrated acceptance, release gate, and continuous iteration`
- Loop target: refresh the governed live commercialization evidence after the admin runtime fix and determine whether the remaining `S08` blocker is still product/runtime truth or only governed release hygiene
- Boundaries:
  - `artifacts/runtime/dev-s08-telemetry-check/run/live-commercial-governance-replay.ps1`
  - `artifacts/runtime/dev-s08-telemetry-check/release-input/*local-governance-pass-replay*`
  - `docs/release/release-telemetry-export-latest.json`
  - `docs/release/release-telemetry-snapshot-latest.json`
  - `docs/release/slo-governance-latest.json`
  - `docs/release/release-window-snapshot-latest.json`
  - `docs/release/release-sync-audit-latest.json`
  - `docs/review/2026-04-11-commercial-s08-live-governance-pass-replay-review.md`
  - `docs/release/2026-04-11-v0.1.51-commercial-s08-live-governance-pass-replay.md`

## Root Cause Investigation

- the prior loop assumption that this host still could not refresh governed live latest artifacts is no longer the best root-cause statement
- fresh investigation showed a narrower blocker:
  - the source-backed replay flow itself can launch successfully from the current host
  - the helper failed only when it hit `ConvertFrom-Json -Depth` while generating supplemental/live summary artifacts
  - current PowerShell does not support that parameter
- after closing that host-tooling mismatch, the governed replay could be rerun truthfully on the same host

## Changes

- added `ConvertFrom-JsonCompat` to the replay helper and replaced the incompatible JSON parsing calls
- replayed the source-backed admin, gateway, and portal stack against the repo-local `dev` bootstrap database
- generated fresh pass-replay evidence and supplemental targets under `artifacts/runtime/dev-s08-telemetry-check/release-input`
- materialized isolated telemetry export, telemetry snapshot, and SLO evidence from that replay
- promoted the same governed truth to the latest release telemetry artifacts under `docs/release`
- refreshed `docs/release/release-window-snapshot-latest.json` and `docs/release/release-sync-audit-latest.json` after this loop backwrite so the final gate replay reads fresh release-governance truth from the same session

## Verification

- helper red/green:
  - `Get-Content ...supplemental-targets-local-governance-fail-replay.json -Raw | ConvertFrom-Json -Depth 32`
  - `Invoke-Expression (Get-Content ...live-commercial-governance-replay.ps1 -Raw); $payload = ConvertFrom-JsonCompat -Text (...)`
- fresh governed live replay:
  - `Invoke-Expression (Get-Content 'artifacts/runtime/dev-s08-telemetry-check/run/live-commercial-governance-replay.ps1' -Raw); Invoke-LiveCommercialGovernanceReplay -DatabaseUrl 'sqlite://D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/artifacts/runtime/dev-s08-telemetry-check/data/sdkwork-api-router-dev.db'`
- isolated gate proof:
  - `node scripts/release/materialize-release-telemetry-export.mjs ...`
  - `node scripts/release/materialize-release-telemetry-snapshot.mjs ...`
  - `node scripts/release/materialize-slo-governance-evidence.mjs ...`
  - `node scripts/release/slo-governance.mjs --format json --evidence artifacts/runtime/dev-s08-telemetry-check/release-input/slo-governance-local-governance-pass-replay.json`
- latest gate proof:
  - `node scripts/release/slo-governance.mjs --format json`
  - `node scripts/release/run-release-governance-checks.mjs --format json`

## Result

- the governed live commercialization evidence chain is now fresh again on the current host
- `release-slo-governance` now passes all `14` baseline targets from live replay truth
- the overall `S08` release gate still remains `no-go`, but only because `release-sync-audit` still fails

## Exit

- Step result: `conditional-go`
- Reason:
  - repo-local commercialization runtime and governed SLO truth are now closed
  - final release closure remains blocked by governed release-sync hygiene rather than by an unresolved `S08` product/runtime defect
