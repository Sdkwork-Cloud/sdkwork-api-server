# Release Windows Installed Runtime Smoke Parity

> Date: 2026-04-08
> Goal: close the remaining Windows release-truth gap described by `docs/架构/143-*` and `docs/架构/144-*` by promoting Windows installed-runtime proof into the governed release workflow.

## 1. Problem

- Native release already built Windows service binaries and desktop bundles.
- Unix release lanes already proved installed-runtime boot, health, and shutdown from a real install layout.
- Windows release lanes still lacked equivalent workflow proof, persisted evidence, and attestation coverage.

## 2. Design

1. Build Windows native service binaries.
2. Build admin and portal desktop artifacts.
3. Materialize an installed runtime home from real Windows release outputs.
4. Rewrite `config/router.env` to loopback ports.
5. Run installed `start.ps1`.
6. Probe:
   - `/api/v1/health`
   - `/api/admin/health`
   - `/api/portal/health`
7. Run installed `stop.ps1`.
8. Persist JSON smoke evidence under `artifacts/release-governance/`.
9. Upload the smoke evidence as a governed artifact.
10. Generate provenance attestation for the Windows smoke evidence.
11. Include the Windows smoke evidence in repository-owned attestation verification.

## 3. Script Contract

- Entry: `scripts/release/run-windows-installed-runtime-smoke.mjs`
- Required inputs:
  - `--platform windows`
  - `--arch <x64|arm64>`
  - `--target <triple>`
- Optional inputs:
  - `--runtime-home <path>`
  - `--evidence-path <path>`
- Behavior:
  - rejects non-Windows lanes
  - reuses `createInstallPlan` and `applyInstallPlan`
  - executes installed `start.ps1` / `stop.ps1` through `powershell.exe -NoProfile -ExecutionPolicy Bypass -File`
  - writes structured success or failure evidence
  - includes log excerpts on failure where available

## 4. Workflow Contract

- Native release workflow must keep:
  - `Run installed native runtime smoke on Windows`
  - `Upload Windows installed runtime smoke evidence`
  - `Generate Windows smoke evidence attestation`
- Ordering rule:
  - after `Build portal desktop release`
  - before `Collect native release assets`
- Evidence rule:
  - path: `artifacts/release-governance/windows-installed-runtime-smoke-${platform}-${arch}.json`
  - artifact name: `release-governance-windows-installed-runtime-smoke-${platform}-${arch}`
- Verification rule:
  - `verify-release-attestations.mjs` must recognize the Windows smoke evidence as a governed subject

## 5. Honest Boundary

- This slice closes repository workflow parity, not hosted execution proof.
- The current restricted local host can still block `Node -> powershell.exe`, so local verification remains contract-level and fallback-safe.
- The next real-world proof point is the first hosted Windows release run that emits governed smoke evidence and a verifiable attestation.

## 6. Remaining Closure

- Still open:
  - first hosted Windows smoke evidence collection on a PowerShell-capable runner outside this restricted local host
  - `release-slo-governance` live telemetry input
  - `release-window-snapshot` and `release-sync-audit` live Git child execution on the current host
