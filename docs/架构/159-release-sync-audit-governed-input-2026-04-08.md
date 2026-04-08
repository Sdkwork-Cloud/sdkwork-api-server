# Release Sync Audit Governed Input

> Date: 2026-04-08
> Goal: let release governance consume auditable multi-repository sync facts even when the current host blocks Node child execution of Git.

## 1. Problem

- `release-sync-audit` is a live release-governance lane.
- It checks multiple repositories and remote refs.
- On restricted hosts the lane can fail before any Git fact is collected.
- Before this slice, there was no governed ingress for externally collected sync facts.

## 2. Input Contract

- Entry: `scripts/release/verify-release-sync.mjs`
- Default mode:
  - recompute repository sync facts through live Git
- Governed input mode:
  - `--audit <path>`
  - `--audit-json <json>`
  - `SDKWORK_RELEASE_SYNC_AUDIT_PATH`
  - `SDKWORK_RELEASE_SYNC_AUDIT_JSON`
- Accepted governed payloads:
  - artifact envelope
    - `generatedAt`
    - `source.kind`
    - `summary`
  - raw summary object
    - `releasable`
    - `reports[]`

## 3. Validation Rules

- `summary.releasable`
  - boolean
- `summary.reports`
  - array
- every report must include:
  - `id`
  - `targetDir`
  - `expectedGitRoot`
  - `topLevel`
  - `remoteUrl`
  - `localHead`
  - `remoteHead`
  - `branch`
  - `upstream`
  - `ahead`
  - `behind`
  - `isDirty`
  - `reasons`
  - `releasable`
- `expectedRef`
  - optional string
- report invariants:
  - `ahead` and `behind` are non-negative integers
  - `isDirty` is boolean
  - `reasons` is string array
  - `releasable` must match whether `reasons` is empty
- summary invariant:
  - `summary.releasable` must match whether every report is releasable

## 4. Execution Policy

- If governed input exists and validates:
  - do not spawn Git
  - use the governed summary directly
- If governed input does not exist:
  - run the live multi-repository audit
- The top-level governance runner must pass `env` into the fallback lane so the same governed sync input works both:
  - for direct `verify-release-sync.mjs` execution
  - for replay through `run-release-governance-checks.mjs`

## 5. Honest Boundary

- Governed sync input is not a shortcut around release truth.
- It is an explicit audited ingress contract for facts collected on an allowed host.
- This slice does not say the local host now has live Git capability.
- It says the repository can now consume governed sync facts when host-local Git child execution is blocked.

## 6. Remaining Closure

- `release-slo-governance` is now the only blocked live governance lane in the current host default run.
- Hosted release execution still needs a defined producer and retention policy for governed release-sync artifacts.
- Default local runs must remain blocked until governed inputs are supplied or host policy changes.
