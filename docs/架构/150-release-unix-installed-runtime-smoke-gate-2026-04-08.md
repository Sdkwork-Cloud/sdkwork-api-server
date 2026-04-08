# Release Unix Installed Runtime Smoke Gate

> Date: 2026-04-08
> Goal: promote Unix installed-runtime proof from repository contracts to a real native release workflow gate.

## 1. Problem

- Native release already built real service binaries and desktop bundles.
- The workflow still packaged assets without proving that the built install layout could boot from an installed home.
- This violated the architecture rule that release must validate `install -> start -> health -> stop` instead of only `build -> package`.

## 2. Gate Design

1. Build native service binaries.
2. Build admin and portal native desktop artifacts.
3. Materialize a temporary installed runtime home from real build outputs.
4. Override `router.env` to loopback random ports.
5. Run installed `start.sh`.
6. Probe unified health endpoints:
   - `/api/v1/health`
   - `/api/admin/health`
   - `/api/portal/health`
7. Run installed `stop.sh`.
8. Only then package native release assets.

## 3. Script Contract

- Entry: `scripts/release/run-unix-installed-runtime-smoke.mjs`
- Required inputs:
  - `--platform <linux|macos>`
  - `--arch <x64|arm64>`
  - `--target <triple>`
- Behavior:
  - rejects Windows lanes
  - uses `createInstallPlan` and `applyInstallPlan`
  - rewrites `config/router.env`
  - emits failure context from installed runtime logs

## 4. Why Unix Only In This Slice

- Linux and macOS lanes already share `start.sh` / `stop.sh` and shell-based health semantics.
- Current host evidence still shows Windows child-process execution can be policy-blocked.
- Therefore this slice closes the honest Unix gap first instead of pretending all platforms are equally verified today.

## 5. Non-Goals

- Do not claim local sandbox proof for full built-artifact startup when release binaries were not built locally in this session.
- Do not turn the Unix gate into a repository-fixture-only test again.
- Do not claim Windows installed-runtime parity yet.

## 6. Remaining Closure

- Windows installed-runtime smoke still needs a stable PowerShell release lane.
- Smoke evidence is still step output, not yet a persisted release artifact.
- Live release-truth lanes that depend on Git child execution remain blocked on the current host.
