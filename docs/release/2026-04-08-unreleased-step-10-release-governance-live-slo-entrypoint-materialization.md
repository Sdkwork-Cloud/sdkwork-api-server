# Unreleased - Step 10 Release Governance Live SLO Entrypoint Materialization

- Date: 2026-04-08
- Type: patch
- Summary:
  - updated `run-release-governance-checks.mjs` so the live `release-slo-governance` lane now replays governed materialization instead of only reading a pre-existing evidence file
  - the fallback path now reuses a governed telemetry snapshot when present, otherwise materializes snapshot and SLO evidence from governed telemetry input, then re-runs live evaluation
  - missing upstream telemetry input is now surfaced as `telemetry-input-missing`, which is more precise than the previous generic `evidence-missing`
  - the single governance entrypoint can now turn `release-slo-governance` green when a governed telemetry export input is supplied, without requiring a separate manual pre-step
- Verification:
  - `release-governance-runner.test.mjs`: `10 / 10`
  - `materialize-release-telemetry-snapshot.test.mjs`: `4 / 4`
  - `materialize-slo-governance-evidence.test.mjs`: `5 / 5`
  - `release-slo-governance.test.mjs`: `5 / 5`
  - default governance summary: `6` pass / `3` block / `0` fail
  - governance summary with temporary telemetry export input: `7` pass / `2` block / `0` fail
- Remaining truth:
  - the repository still does not own a real release-time telemetry export producer
  - snapshot/SLO governance artifacts are not yet uploaded as dedicated release-governance artifacts
  - Git-policy-blocked live lanes remain follow-up work
