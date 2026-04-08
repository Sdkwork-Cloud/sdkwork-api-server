# 2026-04-08 Release Window Snapshot Governed Input Review

## 1. Scope

- `scripts/release/compute-release-window-snapshot.mjs`
- `scripts/release/run-release-governance-checks.mjs`
- `scripts/release/release-window-snapshot-contracts.mjs`
- `scripts/release/tests/release-window-snapshot.test.mjs`
- `scripts/release/tests/release-governance-runner.test.mjs`

## 2. Finding

### P1 release-window snapshot still depended only on host-local Node -> Git execution

- `release-window-snapshot` remained blocked on the current host after the Windows runner-policy correction.
- The repository already had a governed-artifact direction for SLO and smoke evidence.
- `compute-release-window-snapshot.mjs` still lacked an equivalent governed ingress for externally collected release-window facts.
- Result: release governance could only return `command-exec-blocked` on denied hosts, even when operators could provide an auditable snapshot artifact from a trusted environment.

## 3. Root Cause

- The earlier implementation assumed release-window facts had to be recomputed locally through Node child execution of Git.
- No artifact or environment-backed input contract existed for:
  - latest release tag
  - commits since latest release
  - working-tree entry count
  - release baseline presence
- `run-release-governance-checks.mjs` fallback replay therefore had no honest way to consume governed release-window facts when Git child execution was denied.

## 4. Changes

- Updated `compute-release-window-snapshot.mjs`
  - added governed input resolution via:
    - `--snapshot <path>`
    - `--snapshot-json <json>`
    - `SDKWORK_RELEASE_WINDOW_SNAPSHOT_PATH`
    - `SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON`
  - accepts either:
    - a governed artifact with `generatedAt`, `source`, and `snapshot`
    - a raw `snapshot` object with the validated release-window shape
  - exports:
    - `resolveReleaseWindowSnapshotInput`
    - `validateReleaseWindowSnapshot`
    - `validateReleaseWindowSnapshotArtifact`
  - short-circuits to governed input without spawning Git when such input is present
- Updated `run-release-governance-checks.mjs`
  - passes `env` into the `release-window-snapshot` fallback path so governed inputs work in the single governance entrypoint
- Hardened contracts and tests
  - `release-window-snapshot-contracts.mjs` now proves governed-input bypass behavior
  - `release-window-snapshot.test.mjs` now covers governed JSON input
  - `release-governance-runner.test.mjs` now proves governed input turns `release-window-snapshot` green through the top-level governance runner

## 5. Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/release-window-snapshot.test.mjs`
  - `5 / 5`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
  - `12 / 12`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs`
  - `1 / 1`
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - default host truth still reports `release-window-snapshot.reason = command-exec-blocked`
- `node scripts/release/run-release-governance-checks.mjs --format json` with `SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON`
  - `release-window-snapshot` passes through the governed-input path

## 6. Current Truth

- This slice closes the governed ingress contract for release-window facts.
- It does not claim that the current local host now allows live Node -> Git execution.
- Without governed input, the lane still blocks honestly with `command-exec-blocked`.
- With governed input, the repository can now consume auditable external release-window facts instead of faking a PASS.

## 7. Next Step

1. Apply the same governed-input pattern to `release-sync-audit`, which is the remaining Git-blocked live lane.
2. Decide how governed release-window artifacts are produced and retained during hosted release execution.
3. Keep the default local run truthful until an allowed host or governed artifact is supplied.
