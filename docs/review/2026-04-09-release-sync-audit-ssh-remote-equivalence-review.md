# 2026-04-09 Release Sync Audit SSH Remote Equivalence Review

## 1. Scope

- `scripts/release/verify-release-sync.mjs`
- `scripts/release/tests/release-sync-audit.test.mjs`
- `scripts/release/tests/materialize-release-sync-audit.test.mjs`
- `scripts/release/tests/release-governance-runner.test.mjs`

## 2. Finding

### P1 release sync audit falsely rejected valid GitHub SSH clones

- release sync audit hard-coded `expectedRemoteUrl` as HTTPS GitHub URLs.
- local operator guidance and current workspace practice use SSH remotes for `git push`.
- result: the audit could report `remote-url-mismatch` even when the clone pointed at the same GitHub repository.

## 3. Root Cause

- `verify-release-sync.mjs` compared `remoteUrl.trim()` against `expectedRemoteUrl` with raw string equality.
- the contract encoded transport syntax, not repository identity.
- that broke parity between:
  - `https://github.com/Sdkwork-Cloud/<repo>.git`
  - `git@github.com:Sdkwork-Cloud/<repo>.git`
  - `ssh://git@github.com/Sdkwork-Cloud/<repo>.git`

## 4. Changes

- added GitHub remote canonicalization before sync-audit comparison.
- kept the audit strict for:
  - wrong owner/repo
  - dirty working tree
  - ahead/behind divergence
  - remote head mismatch
  - remote unverifiable failures
- added a red-first regression proving an SSH GitHub origin is equivalent to the expected HTTPS origin.

## 5. Verification

- red-first:
  - `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs`
    - failed on SSH-origin equivalence before the fix
- green:
  - `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs`
    - `3 / 3`
  - `node --test --experimental-test-isolation=none scripts/release/tests/materialize-release-sync-audit.test.mjs`
    - `3 / 3`
  - `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
    - `16 / 16`
- live governance truth:
  - `node scripts/release/run-release-governance-checks.mjs --format json`
    - `ok=false`
    - `blocked=true`
    - `passingIds=7`
    - `blockedIds=3`
    - `failingIds=[]`

## 6. Current Truth

- this fix removes one false-negative class from release-sync audit.
- this host is still honestly not releasable by default because:
  - `release-window-snapshot` and `release-sync-audit` still hit Node child-Git `EPERM`
  - `release-slo-governance` still lacks governed telemetry input
- shell evidence on this workspace also shows separate real release blockers:
  - `git remote get-url origin` returns the SSH form of the correct repository
  - `git status --short --branch` reports `## main...origin/main [ahead 2]` plus a dirty tree
  - `git ls-remote origin main` currently fails on this host's SSH execution boundary
- therefore URL normalization was necessary, but it does not fabricate green release truth.

## 7. Next Step

1. Keep Step 11/13 focused on governed release inputs, especially telemetry export supply.
2. If blocked-host operator UX remains a priority, add a documented host-side procedure to materialize truthful latest artifacts before running the governance gate.
3. Re-run release sync truth only after the working tree is clean and the branch is no longer ahead.
