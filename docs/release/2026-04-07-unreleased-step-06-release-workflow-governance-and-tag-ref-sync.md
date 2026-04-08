# 2026-04-07 Unreleased Step 06 Release Workflow Governance and Tag Ref Sync

## Summary

This Step `06` slice hardened the release-truth lane without changing product runtime behavior.

## Changes

- expanded release-time GitHub materialization from a single sibling helper to the governed repository set:
  - `sdkwork-core`
  - `sdkwork-ui`
  - `sdkwork-appbase`
  - `sdkwork-im-sdk`
- inserted `Run release governance gate` into both `native-release` and `web-release` after materialization and before dependency installation
- passed repository-specific release refs into CI through:
  - `SDKWORK_API_ROUTER_GIT_REF`
  - `SDKWORK_CORE_GIT_REF`
  - `SDKWORK_UI_GIT_REF`
  - `SDKWORK_APPBASE_GIT_REF`
  - `SDKWORK_IM_SDK_GIT_REF`
- updated `verify-release-sync.mjs` so the main repository can be validated from a detached release tag ref instead of requiring a tracking branch during tag builds
- switched remote ref verification to `ls-remote origin <expectedRef>` and accepted peeled tag output so annotated release tags resolve correctly

## Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release/tests/release-governance-runner.test.mjs scripts/release/tests/release-window-snapshot.test.mjs`

Observed on 2026-04-07:

- all `12` tests passed
- `node scripts/release/run-release-governance-checks.mjs --format json` still blocked live sync truth with `command-exec-blocked`
- shell-verified release window remained:
  - latest tag `release-2026-03-28-8`
  - `16` commits since tag
  - `622` working-tree entries

## Release Decision

- Status: blocked / unpublished
- Reason: live multi-repository sync truth cannot be proven in the current sandbox, so `commit -> push -> GitHub release` remains forbidden
- Carry-forward rule: this note must be folded into the next verified successful GitHub release window
