# 2026-04-11 Commercial S08 Live Governance Pass Replay Review

## Scope

- Architecture reference: `166`, `133`, `03`
- Step reference: `110`
- Loop focus: determine whether the remaining `S08` blocker after the admin runtime fix is still a commercialization/runtime issue, or whether the gate can now be reduced to governed release-sync hygiene only

## Findings

### P0 - the governed live replay now proves the commercial control plane and full SLO baseline pass on the current source-backed runtime

- fresh replay evidence now covers the real source-backed stack rather than only service-local tests:
  - `POST /admin/auth/login`: `200`
  - `GET /admin/billing/summary`: `200`
  - `GET /admin/billing/account-holds`: `200`
  - `GET /admin/billing/request-settlements`: `200`
  - `POST /admin/billing/pricing-lifecycle/synchronize`: `200`
- the replay generated a fresh governed evidence pack and both isolated and latest telemetry artifacts
- `node scripts/release/slo-governance.mjs --format json` now returns `ok: true` with all `14` baseline targets passing
- impact:
  - the prior `release-slo-governance` failure is closed from real live evidence, not from inference
  - the repo-local commercialization/runtime blocker is no longer open

### P1 - the last host-side replay blocker was a PowerShell helper incompatibility, not missing child-process capability

- the current host can successfully launch the replayed stack through the helper flow
- the failing edge was narrower:
  - `live-commercial-governance-replay.ps1` used `ConvertFrom-Json -Depth`
  - that parameter is unsupported in the current PowerShell host
  - the helper therefore failed only while writing supplemental/live summary artifacts
- fix:
  - added `ConvertFrom-JsonCompat`
  - reused the compatible path for response parsing, error parsing, and supplemental-target template loading
- impact:
  - the replay helper now completes end-to-end on the current host and produces governed evidence reliably

### P1 - the overall release gate is now reduced to release-sync hygiene only

- current governed gate posture:
  - `release-window-snapshot`: `pass`
  - `release-slo-governance`: `pass`
  - `release-sync-audit`: `fail`
- current failing release-sync reasons remain concrete and independent of product/runtime truth:
  - dirty worktrees
  - unsynced branch state
  - remote unverifiability
  - `sdkwork-core` root/remote mismatch
- impact:
  - `S08` still cannot claim release closure
  - but the remaining blocker is now externalized to governed release hygiene rather than commercialization capability

## Fix Closure

- closed the PowerShell replay-helper compatibility bug
- closed the governed live replay freshness gap for release telemetry and SLO artifacts
- narrowed the final `S08` `no-go` to `release-sync-audit` only

## Verification

- replay helper red/green:
  - `Get-Content ...supplemental-targets-local-governance-fail-replay.json -Raw | ConvertFrom-Json -Depth 32`
  - `Invoke-Expression (Get-Content ...live-commercial-governance-replay.ps1 -Raw); $payload = ConvertFrom-JsonCompat -Text (...)`
- live governed replay:
  - `Invoke-Expression (Get-Content 'artifacts/runtime/dev-s08-telemetry-check/run/live-commercial-governance-replay.ps1' -Raw); Invoke-LiveCommercialGovernanceReplay -DatabaseUrl 'sqlite://D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/artifacts/runtime/dev-s08-telemetry-check/data/sdkwork-api-router-dev.db'`
- isolated evidence proof:
  - `node scripts/release/materialize-release-telemetry-export.mjs ...`
  - `node scripts/release/materialize-release-telemetry-snapshot.mjs ...`
  - `node scripts/release/materialize-slo-governance-evidence.mjs ...`
  - `node scripts/release/slo-governance.mjs --format json --evidence artifacts/runtime/dev-s08-telemetry-check/release-input/slo-governance-local-governance-pass-replay.json`
- latest gate proof:
  - `node scripts/release/slo-governance.mjs --format json`
  - `node scripts/release/run-release-governance-checks.mjs --format json`

## Residual Risks

- any further doc/release backwrite still requires a fresh `release-window-snapshot-latest.json`
- `release-sync-audit` remains `fail` until governed repo hygiene and remote verification are resolved or explicitly accepted as an external blocker

## Exit

- Step result: `conditional-go`
- Reason:
  - commercialization runtime truth and governed SLO truth are now closed
  - final release closure remains blocked by governed release-sync hygiene outside the commercial product/runtime path
