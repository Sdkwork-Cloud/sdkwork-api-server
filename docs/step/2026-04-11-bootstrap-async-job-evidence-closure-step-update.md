# 2026-04-11 Bootstrap Async Job Evidence Closure Step Update

## What Changed

- Hardened bootstrap validation for `async_job_attempts`, `async_job_assets`, and `async_job_callbacks`.
- Added parent-lifecycle invariants for attempts:
  - `claimed_at_ms` must not be earlier than parent `async_jobs.started_at_ms`
  - `finished_at_ms` must not be later than parent `async_jobs.completed_at_ms`
- Added asset evidence invariants:
  - `async_job_assets.created_at_ms` must not be earlier than parent `async_jobs.created_at_ms`
  - `async_job_assets.storage_key` must contain the parent `job_id`
  - `async_job_assets.storage_key` must stay under `tenant-{tenant_id}/jobs/`
  - `async_job_assets.download_url`, when present, must contain the parent `job_id`
- Added callback evidence invariants:
  - `async_job_callbacks.received_at_ms` must not be earlier than parent `async_jobs.created_at_ms`
  - `event_type = job.completed` must not arrive before parent `async_jobs.completed_at_ms`
  - callback payload `status`, when present, must match the parent job status
  - callback payload `provider`, when present, must match the parent job provider identity

## Why This Matters

- Async job seed data is part of the commercial install-ready operating surface, not just demo filler.
- Previous validation already covered:
  - job and attempt local timestamp ordering
  - attempt external job id alignment
  - callback payload `job_id` alignment
  - processed callback timestamp presence and receive/process ordering
- That still left evidence gaps:
  - attempts could drift outside the parent job lifecycle window
  - assets could point at storage paths outside the parent tenant/job scope
  - callbacks could arrive before the parent job lifecycle they claim to represent
  - callback payload status/provider could silently diverge from the parent job
- In a commercial bootstrap pack, those gaps weaken operator trust in seeded job history, asset lineage, and callback replay evidence.

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using profile + updates + last-wins collapse for:
  - `data/jobs/*.json`
  - additive `data/updates/*.json`
- Audit result:
  - `PROFILE=prod ATTEMPT_CLAIM_BEFORE_JOB_START=0 ATTEMPT_FINISH_AFTER_JOB_COMPLETE=0 ASSET_BEFORE_JOB_CREATE=0 ASSET_STORAGE_JOB_MISMATCH=0 ASSET_STORAGE_TENANT_MISMATCH=0 ASSET_DOWNLOAD_JOB_MISMATCH=0 CALLBACK_BEFORE_JOB_CREATE=0 COMPLETED_CALLBACK_BEFORE_JOB_COMPLETE=0 CALLBACK_PAYLOAD_STATUS_MISMATCH=0 CALLBACK_PAYLOAD_PROVIDER_MISMATCH=0`
  - `PROFILE=dev ATTEMPT_CLAIM_BEFORE_JOB_START=0 ATTEMPT_FINISH_AFTER_JOB_COMPLETE=0 ASSET_BEFORE_JOB_CREATE=0 ASSET_STORAGE_JOB_MISMATCH=0 ASSET_STORAGE_TENANT_MISMATCH=0 ASSET_DOWNLOAD_JOB_MISMATCH=0 CALLBACK_BEFORE_JOB_CREATE=0 COMPLETED_CALLBACK_BEFORE_JOB_COMPLETE=0 CALLBACK_PAYLOAD_STATUS_MISMATCH=0 CALLBACK_PAYLOAD_PROVIDER_MISMATCH=0`

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger async-job evidence contract.
- This step turns current job artifacts and callback evidence into explicit bootstrap guarantees.

## Test Coverage Added

- async job attempt rejects `claimed_at_ms` earlier than parent job `started_at_ms`
- async job attempt rejects `finished_at_ms` later than parent job `completed_at_ms`
- async job asset rejects `created_at_ms` earlier than parent job `created_at_ms`
- async job asset rejects storage key outside parent tenant scope
- async job callback rejects `received_at_ms` earlier than parent job `created_at_ms`
- completed async job callback rejects `received_at_ms` earlier than parent job `completed_at_ms`
- async job callback rejects payload status mismatch with parent job
- async job callback rejects payload provider mismatch with parent job

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_async_job_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- Future job bootstrap packs should treat storage paths, callback payloads, and parent lifecycle timestamps as a closed evidence set instead of loosely related metadata.
- If later product flows add non-completion callback types or provider aliasing variants, extend the contract explicitly rather than weakening the current parent/child evidence guarantees.
