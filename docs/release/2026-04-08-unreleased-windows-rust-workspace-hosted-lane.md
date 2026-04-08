# 2026-04-08 Unreleased Windows Rust Workspace Hosted Lane

## 1. Iteration Context

- Wave / Step: `Wave B / Step 06` support lane
- Primary mode: `verification-blocker-clearance`
- Current state classification: `未闭环执行中`

## 2. Top 3 Candidate Actions

1. Add a hosted Windows `workspace` verification lane without slowing the default PR matrix.
2. Keep all verification local-only and defer hosted proof collection.
3. Expand immediately into broader Step 06 business-workspace feature review.

Action `1` was selected because the current highest-value gap after local Windows workspace stabilization was the absence of a repository-owned hosted path for the same evidence.

## 3. Actual Changes

- updated `.github/workflows/rust-verification.yml`
  - kept the default split-package matrix on `ubuntu-latest`
  - added a manual `workflow_dispatch` job `rust-verification-windows-workspace`
  - the new lane runs on `windows-latest`
  - the new lane executes `node scripts/check-rust-verification-matrix.mjs --group workspace`
- updated `scripts/rust-verification-workflow.test.mjs`
  - added a red-to-green contract for the new hosted Windows workspace lane
- updated `docs/review/2026-04-06-rust-verification-matrix.md`
  - documented that `workspace` is now available both as a local deep gate and as a manual hosted Windows lane
- updated `docs/review/2026-04-06-application-review.md`
  - recorded the workflow contract proof alongside the local workspace verification evidence

## 4. Verification

- `node --test --experimental-test-isolation=none scripts/rust-verification-workflow.test.mjs`
- `node --test --experimental-test-isolation=none scripts/check-rust-verification-matrix.test.mjs scripts/rust-verification-workflow.test.mjs scripts/dev/tests/windows-rust-toolchain-guard.test.mjs`

## 5. Architecture / Delivery Impact

- the repository now has a controlled path to collect hosted Windows full-workspace Rust evidence
- the default pull-request verification cost remains bounded because the heavy Windows workspace lane is manual-only
- this improves release-truth readiness but does not yet prove a successful hosted execution

## 6. Risks / Limits

- no hosted `windows-latest` run was executed in this sandbox session
- Linux/macOS runtime or packaging evidence is still absent
- Step 06 business-workspace closure remains open beyond the verification lane

## 7. Next Entry

1. Record the first hosted execution result for `workflow_dispatch group=workspace`.
2. Continue the highest-value Step 06 closure lane after hosted verification truth is materially improved.
