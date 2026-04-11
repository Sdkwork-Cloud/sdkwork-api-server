# 2026-04-10 Commercial S08 Managed Telemetry Availability Proof Review

## Scope

- Architecture reference: `166`
- Step reference: `110`
- Loop focus: verify whether the `release-slo-governance` blocker is still "no live local telemetry" or has narrowed to the remaining non-availability SLO coverage gap

## Findings

### P0 - local managed telemetry exists, but the governed SLO chain still breaks at missing non-availability targets

- managed services already answer:
  - `GET http://127.0.0.1:9980/health`
  - `GET http://127.0.0.1:9981/admin/health`
  - `GET http://127.0.0.1:9982/portal/health`
- authenticated `GET /metrics` on `9980 / 9981 / 9982` returns real Prometheus text, so local availability telemetry is not the missing input anymore
- the repo can now materialize a truthful release telemetry export from those live metrics, but snapshot derivation still fails at:
  - `release telemetry snapshot is missing target gateway-non-streaming-success-rate`
- impact:
  - the release telemetry lane is only partially closable locally
  - the release blocker has narrowed to truthful `supplemental.targets` coverage for the remaining non-availability SLO targets

### P1 - the earlier `8080 / 8081 / 8082` probe was incomplete evidence, not wrong evidence

- raw standalone defaults are still absent on `8080 / 8081 / 8082`
- however, managed runtime defaults on this host are live on `9980 / 9981 / 9982`
- an isolated `start-dev.ps1` replay failed during warm-up because `portal-api-service.exe` in the shared Windows target directory was locked, and concurrent process inspection showed the managed service executables were already running
- impact:
  - startup replay failure here does not support a claim that managed runtime is impossible
  - it supports the opposite conclusion: the host already carries managed service processes and live observability

## Fix Closure

- corrected the S08 blocker wording from "no local telemetry" to "local availability telemetry exists, but governed SLO coverage remains incomplete"
- backwrote `110`, `166`, changelog, and the new release note to the same narrower release diagnosis
- kept the final commercialization posture honest at `no-go`

## Verification

- managed-service probes:
  - `Invoke-WebRequest http://127.0.0.1:9980/health`
  - `Invoke-WebRequest http://127.0.0.1:9981/admin/health`
  - `Invoke-WebRequest http://127.0.0.1:9982/portal/health`
  - `Invoke-WebRequest -Headers @{ Authorization = 'Bearer local-dev-metrics-token' } http://127.0.0.1:9980/metrics`
  - `Invoke-WebRequest -Headers @{ Authorization = 'Bearer local-dev-metrics-token' } http://127.0.0.1:9981/metrics`
  - `Invoke-WebRequest -Headers @{ Authorization = 'Bearer local-dev-metrics-token' } http://127.0.0.1:9982/metrics`
- startup replay:
  - `powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\\bin\\start-dev.ps1 -WaitSeconds 240 -GatewayBind 127.0.0.1:19980 -AdminBind 127.0.0.1:19981 -PortalBind 127.0.0.1:19982 -WebBind 127.0.0.1:19983`
- governed export/snapshot replay:
  - `node scripts/release/materialize-release-telemetry-export.mjs --gateway-prometheus ... --admin-prometheus ... --portal-prometheus ...`
  - `node scripts/release/materialize-release-telemetry-snapshot.mjs --export artifacts/runtime/dev-s08-telemetry-check/release-input/release-telemetry-export-live.json`

## Residual Risks

- default governed `release-telemetry-snapshot-latest.json` and `slo-governance-latest.json` are still absent
- `release-sync-audit` remains independently negative even if telemetry later closes
- a future loop still needs truthful `supplemental.targets`, not synthetic placeholders

## Exit

- Step result: `no-go`
- Reason:
  - local observability is partially proven
  - governed release truth still lacks the remaining non-availability SLO evidence
