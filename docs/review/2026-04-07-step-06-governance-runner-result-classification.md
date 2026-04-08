# 2026-04-07 Step 06 Governance Runner Result Classification Review

## Scope

This review slice continued Wave `B` / Step `06` on the release-governance lane and focused on top-level release-truth classification.

Primary target in this round:

- make the governance runner distinguish environment-blocked lanes from real failing lanes without changing release gates

Execution boundary:

- do not change product runtime behavior
- do not weaken release blocking rules
- do not commit, push, tag, or publish a GitHub release

## Decision Ledger

- Date: `2026-04-07`
- Wave / Step: `B / 06`
- Primary mode: `release-gate-hardening`
- Previous mode: `release-gate-hardening`
- Strategy switch: no

### Top 3 Candidate Actions

1. Add explicit blocked/failing/passing summary fields to the governance runner and keep existing lane details intact.
   - `Priority Score: 93`
   - makes operator output actionable without weakening any gate
2. Keep only raw per-lane results and ask operators to infer blocked status from nested JSON.
   - `Priority Score: 41`
   - rejected because it keeps the most important release decision hidden in payload details
3. Stop on the first blocked lane and remove aggregate output.
   - `Priority Score: 22`
   - rejected because later lanes still provide release-truth evidence

### Chosen Action

Action 1 was selected because the release-governance runner is the single release-truth entry point and should expose a concise, machine-readable decision summary.

## TDD Evidence

Red first:

- updated `scripts/release/tests/release-governance-runner.test.mjs`
- required:
  - `blocked`
  - `passingIds`
  - `blockedIds`
  - `failingIds`
- ran:
  - `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
- observed expected failures:
  - the new summary fields were `undefined`

Green after minimal implementation:

- updated `scripts/release/run-release-governance-checks.mjs`
- re-ran the same test command
- result:
  - pass

## Implemented Fixes

- `scripts/release/run-release-governance-checks.mjs`
  - added JSON-safe result classification helpers
  - detects blocked release-window snapshot payloads
  - detects blocked release-sync audit payloads
  - returns top-level `blocked`, `passingIds`, `blockedIds`, and `failingIds`
  - keeps `ok` and `results` stable for existing callers
  - prints `FAIL` instead of `BLOCK` for non-blocked failing lanes in text mode
- `scripts/release/tests/release-governance-runner.test.mjs`
  - now verifies pure failing-lane aggregation
  - now verifies mixed blocked-versus-failing classification

## Verification Evidence

- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release/tests/release-governance-runner.test.mjs scripts/release/tests/release-window-snapshot.test.mjs`
- `node scripts/release/run-release-governance-checks.mjs --format json`
- shell Git facts on 2026-04-07:
  - latest tag `release-2026-03-28-8`
  - commits since tag `16`
  - working-tree entries `631`

Observed result:

- the targeted release-governance suite passed with `16` tests and `0` failures
- the live governance runner now reports top-level blocked/failing/passing classification
- the current sandbox still blocks both live snapshot truth and live sync truth

## Current Assessment

### Closed In This Slice

- blocked and failing governance lanes are now distinguishable at the summary layer
- machine-readable release-truth output is now easier to consume in CI and operator workflows

### Still Open

- live repository sync truth is still blocked in this sandbox
- live snapshot truth is still blocked in this sandbox
- no commit / push / GitHub release is authorized

## Next Slice Recommendation

1. Surface the new summary fields in operator-facing release docs once the release README encoding risk is handled safely.
2. Run the same governance command in an environment where Git child-process execution is allowed.
3. Only after both live snapshot and live sync lanes are green should release publication be reconsidered.
