# 2026-04-10 Bootstrap Async Job Lineage Closure Step Update

## What Changed

- Hardened bootstrap validation so `async_job_attempts.external_job_id` must match the parent `async_jobs.external_job_id` when both are present.
- Hardened bootstrap validation so `async_job_attempts.created_at_ms` cannot be earlier than the parent job's `created_at_ms`.
- Hardened bootstrap validation so `async_job_callbacks.payload_json.job_id`, when present, must match the parent `async_jobs.job_id`.
- Added regression tests for:
  - async job attempts whose external job id drifts from the parent job
  - async job attempts that appear before the parent job exists
  - async job callbacks whose payload points at another job id

## Why This Matters

- `async_jobs`, `async_job_attempts`, and `async_job_callbacks` together form one execution lineage, not three loosely related seed lists.
- Commercial bootstrap data must preserve parent-child identity across retry attempts and callback evidence, otherwise seeded environments can load execution history that operators cannot trust.
- Tightening these relations improves idempotent bootstrap safety without changing the commercial seed model itself.

## Data Impact

- No repository `/data` seed pack required changes.
- A direct lineage audit across `data/jobs/*.json` confirmed there is no existing drift in:
  - `attempt.external_job_id -> job.external_job_id`
  - `attempt.created_at_ms >= job.created_at_ms`
  - `callback.payload_json.job_id -> callback.job_id`

## Design Notes

- This step intentionally did **not** enforce `async_job_assets.created_at_ms <= async_jobs.updated_at_ms`.
- Real repository seed data already contains legitimate assets created after the parent job's latest update timestamp, so that upper-bound check would over-constrain valid bootstrap evidence.
- The hardening here only promotes lineage rules that are already stable in shipped commercial seed data.

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_async_job_attempt_with_mismatched_external_job_id -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_async_job_attempt_created_before_parent_job -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_async_job_callback_with_mismatched_payload_job_id -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- The next safe closure candidate is `async_job_attempts.claimed_at_ms / finished_at_ms` against parent `started_at_ms / completed_at_ms`, but only after a dedicated audit confirms there are no legitimate delayed-finalization patterns in future seed packs.
