# 2026-04-10 Bootstrap Async Job Provider Executability Closure Step Update

## What Changed

- Hardened bootstrap validation so `async_jobs.provider_id` must resolve to an executable provider account in the current deployment.
- Added a regression test that proves bootstrap rejects async jobs pointing at a catalog-valid provider with no executable account.

## Why This Matters

- `async_jobs` are execution evidence, not just configuration.
- Before this change, bootstrap could accept jobs that claimed work ran on a provider that was present in catalog metadata but not actually runnable in the deployment.
- That state is commercially unsafe because seeded environments could look operational while carrying execution history that the runtime could never reproduce or continue.

## Scope Choice

- `AsyncJobRecord.tenant_id` is numeric and the bootstrap layer still does not have a stable mapping to workspace string tenant ids.
- For that reason, this step enforces global executability rather than tenant-scoped executability.
- This matches the same safety line already used for `request_meter_facts.provider_code`.

## Data Impact

- No `/data` changes were needed.
- Repository job seed data already passed the stricter closure, including official, proxy, and local provider job records under `data/jobs/`.

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_async_job_without_executable_provider_account -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_job_attempt_with_missing_job -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`

## Follow-Up

- If bootstrap later gains a stable numeric-to-workspace tenant mapping for job ownership, `async_jobs.provider_id` can be tightened from global executability to tenant-accessible executability.
