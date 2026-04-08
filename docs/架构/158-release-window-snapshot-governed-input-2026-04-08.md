# Release Window Snapshot Governed Input

> Date: 2026-04-08
> Goal: let release governance consume auditable release-window facts even when the current host blocks Node child execution of Git.

## 1. Problem

- `release-window-snapshot` is part of live release governance truth.
- Some hosts, including the current local host, deny Node -> Git child execution.
- Before this slice, the lane had only one execution path:
  - recompute facts locally through Git
- That made the lane truthful but unnecessarily closed to governed external facts collected on an allowed host.

## 2. Input Contract

- Entry: `scripts/release/compute-release-window-snapshot.mjs`
- Default mode:
  - recompute release-window facts from Git
- Governed input mode:
  - `--snapshot <path>`
  - `--snapshot-json <json>`
  - `SDKWORK_RELEASE_WINDOW_SNAPSHOT_PATH`
  - `SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON`
- Accepted governed payloads:
  - artifact envelope
    - `generatedAt`
    - `source.kind`
    - `snapshot`
  - raw snapshot object
    - `latestReleaseTag`
    - `commitsSinceLatestRelease`
    - `workingTreeEntryCount`
    - `hasReleaseBaseline`

## 3. Validation Rules

- `snapshot.latestReleaseTag`
  - string
- `snapshot.commitsSinceLatestRelease`
  - non-negative integer or `null`
- `snapshot.workingTreeEntryCount`
  - non-negative integer
- `snapshot.hasReleaseBaseline`
  - boolean
- if `hasReleaseBaseline` is `false`:
  - `latestReleaseTag` must be empty
  - `commitsSinceLatestRelease` must be `null`
- artifact mode additionally requires:
  - non-empty `generatedAt`
  - `source` object
  - non-empty `source.kind`

## 4. Execution Policy

- If governed input exists and validates:
  - do not spawn Git
  - return the governed snapshot directly
- If governed input does not exist:
  - run the live Git path
- If the live Git path hits `EPERM` or `EACCES`:
  - return blocked `command-exec-blocked`
- The top-level governance runner must pass `env` into the fallback lane so the same governed input works both:
  - when invoking `compute-release-window-snapshot.mjs` directly
  - when replaying through `run-release-governance-checks.mjs`

## 5. Honest Boundary

- Governed input is not a fake PASS.
- It is a separate, explicit evidence ingress contract.
- This slice does not say the local host now has live Git capability.
- It says the repository can accept auditable release-window facts from a trusted producer when local Git child execution is blocked.

## 6. Remaining Closure

- The next adjacent gap is `release-sync-audit`, which still lacks the same governed ingress pattern.
- Hosted release execution still needs a defined producer and retention policy for release-window artifacts.
- Default local runs must remain blocked until governed input is supplied or host policy changes.
