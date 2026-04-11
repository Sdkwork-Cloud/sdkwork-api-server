# 2026-04-10 S08 Managed Telemetry Availability Proof Step Update

## Scope

- Step: `S08 integrated acceptance, release gate, and continuous iteration`
- Loop target: verify whether the remaining `release-slo-governance` blocker is still "no live telemetry input at all" or has narrowed to a smaller governed evidence gap
- Boundaries:
  - `artifacts/runtime/dev-s08-telemetry-check/release-input/*`
  - `docs/step/110-S08-集成验收-发布门禁与持续迭代-2026-04-10.md`
  - `docs/review/*`
  - `docs/release/*`
  - `docs/架构/166-*`

## Root Cause Investigation

- raw standalone truth:
  - `127.0.0.1:8080 / 8081 / 8082` remain unreachable on this host
  - that only proves the raw standalone defaults are absent
- managed host truth:
  - live `gateway`, `admin`, and `portal` services are already reachable on `127.0.0.1:9980 / 9981 / 9982`
  - authenticated `/metrics` probes succeed on all three managed endpoints with `Bearer local-dev-metrics-token`
- managed startup replay:
  - an isolated `start-dev.ps1` launch with custom ports and `SDKWORK_ROUTER_DEV_HOME=artifacts/runtime/dev-s08-telemetry-check` reached Windows backend warm-up
  - warm-up then failed because `bin/.sdkwork-target-vs2022/debug/portal-api-service.exe` could not be removed (`os error 5`)
  - concurrent process inspection showed live `admin-api-service`, `gateway-service`, and `portal-api-service` executables already running, so the shared build-output lock is consistent with pre-existing managed runtime activity
- governed materialization truth:
  - live `/metrics` content was captured into:
    - `artifacts/runtime/dev-s08-telemetry-check/release-input/gateway.prom`
    - `artifacts/runtime/dev-s08-telemetry-check/release-input/admin.prom`
    - `artifacts/runtime/dev-s08-telemetry-check/release-input/portal.prom`
  - `materialize-release-telemetry-export.mjs` succeeded and produced:
    - `artifacts/runtime/dev-s08-telemetry-check/release-input/release-telemetry-export-live.json`
  - `materialize-release-telemetry-snapshot.mjs` then failed on:
    - `release telemetry snapshot is missing target gateway-non-streaming-success-rate`

## Changes

- replaced the earlier "no reachable localhost telemetry" blanket conclusion with a narrower and more truthful diagnosis
- established that local managed observability is already present for the three availability targets
- established that the remaining repo-local closure gap is the non-availability portion of the SLO baseline:
  - truthful `supplemental.targets`
  - plus the default governed latest snapshot / SLO evidence writeback that depends on them
- updated `110`, `166`, changelog, release note, and review records to align on the sharper blocker wording

## Verification

- live process evidence:
  - `Get-Process | Where-Object { $_.ProcessName -like '*portal-api-service*' -or $_.ProcessName -like '*admin-api-service*' -or $_.ProcessName -like '*gateway-service*' -or $_.ProcessName -like '*cargo*' }`
- raw defaults still absent:
  - `Invoke-WebRequest http://127.0.0.1:8080/metrics`
  - `Invoke-WebRequest http://127.0.0.1:8081/metrics`
  - `Invoke-WebRequest http://127.0.0.1:8082/metrics`
- managed health and metrics:
  - `Invoke-WebRequest http://127.0.0.1:9980/health`
  - `Invoke-WebRequest http://127.0.0.1:9981/admin/health`
  - `Invoke-WebRequest http://127.0.0.1:9982/portal/health`
  - `Invoke-WebRequest -Headers @{ Authorization = 'Bearer local-dev-metrics-token' } http://127.0.0.1:9980/metrics`
  - `Invoke-WebRequest -Headers @{ Authorization = 'Bearer local-dev-metrics-token' } http://127.0.0.1:9981/metrics`
  - `Invoke-WebRequest -Headers @{ Authorization = 'Bearer local-dev-metrics-token' } http://127.0.0.1:9982/metrics`
- startup replay:
  - `powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\\bin\\start-dev.ps1 -WaitSeconds 240 -GatewayBind 127.0.0.1:19980 -AdminBind 127.0.0.1:19981 -PortalBind 127.0.0.1:19982 -WebBind 127.0.0.1:19983`
- governed chain replay:
  - `node scripts/release/materialize-release-telemetry-export.mjs --gateway-prometheus ... --admin-prometheus ... --portal-prometheus ...`
  - `node scripts/release/materialize-release-telemetry-snapshot.mjs --export artifacts/runtime/dev-s08-telemetry-check/release-input/release-telemetry-export-live.json`

## Result

- `release-slo-governance` is still not closable from current repo truth
- but the blocker is now more accurate:
  - live managed availability telemetry exists locally
  - the governed export step is locally executable
  - the remaining failure point is missing truthful `supplemental.targets` for the non-availability SLO baseline
- `release-window-snapshot` remains `pass`
- `release-sync-audit` remains `fail`

## Exit

- Step result: `no-go`
- Reason:
  - `S08` is no longer blocked by a total lack of local metrics
  - it is blocked by incomplete governed telemetry coverage plus existing release-sync hygiene failure
