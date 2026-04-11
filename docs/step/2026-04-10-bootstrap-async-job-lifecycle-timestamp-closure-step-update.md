# 2026-04-10 Bootstrap Async Job Lifecycle Timestamp Closure Step Update

## What Changed

- Hardened bootstrap lifecycle validation for `async_jobs`, `async_job_attempts`, and `async_job_callbacks`.
- Added status-driven timestamp invariants:
  - `async_jobs.status = succeeded` must declare both `started_at_ms` and `completed_at_ms`
  - `async_job_attempts.status = succeeded` must declare both `claimed_at_ms` and `finished_at_ms`
  - `async_job_callbacks.status = processed` must declare `processed_at_ms`

## Why This Matters

- Async job bootstrap data is part of the install-ready operational story, not just demo decoration.
- Previous validation already covered:
  - job and attempt creation/update ordering
  - attempt finish after claim
  - callback processed time not earlier than receive time
  - callback payload lineage and dedupe hygiene
- That still left lifecycle gaps:
  - a succeeded job could omit the timestamps that prove it actually started or completed
  - a succeeded attempt could omit the worker claim or finish timestamps
  - a processed callback could omit the time when processing finished
- In a commercial bootstrap pack, those gaps weaken operator trust in job history, callback replay reasoning, and runtime incident analysis.

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using profile + updates + last-wins collapse for:
  - `data/jobs/*.json`
  - additive `data/updates/*.json`
- Audit result:
  - `PROFILE=prod JOB_SUCCEEDED_MISSING_STARTED=0 JOB_SUCCEEDED_MISSING_COMPLETED=0 JOB_SUCCEEDED_PROGRESS_NOT_100=0 ATTEMPT_SUCCEEDED_MISSING_CLAIMED=0 ATTEMPT_SUCCEEDED_MISSING_FINISHED=0 CALLBACK_PROCESSED_MISSING_TS=0`
  - `PROFILE=dev JOB_SUCCEEDED_MISSING_STARTED=0 JOB_SUCCEEDED_MISSING_COMPLETED=0 JOB_SUCCEEDED_PROGRESS_NOT_100=0 ATTEMPT_SUCCEEDED_MISSING_CLAIMED=0 ATTEMPT_SUCCEEDED_MISSING_FINISHED=0 CALLBACK_PROCESSED_MISSING_TS=0`

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger async-job lifecycle timestamp contract.
- This step formalizes the lifecycle evidence that the bootstrap data was already carrying implicitly.

## Test Coverage Added

- succeeded async job rejects missing `started_at_ms`
- succeeded async job rejects missing `completed_at_ms`
- succeeded async job attempt rejects missing `claimed_at_ms`
- succeeded async job attempt rejects missing `finished_at_ms`
- processed async job callback rejects missing `processed_at_ms`

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_async_job -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- Future job-domain bootstrap packs should treat status and timestamps as a closed lifecycle ledger, not optional UI hints.
- If richer terminal states are introduced later, extend the status contract explicitly instead of weakening the current succeeded/processed guarantees.
