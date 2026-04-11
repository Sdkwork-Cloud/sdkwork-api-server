# 2026-04-10 Bootstrap Async Job Account Closure Step Update

## What Changed

- Hardened bootstrap validation so `async_jobs.account_id` must reference an existing account.
- Hardened bootstrap validation so every seeded async job that binds `account_id` must match the owning account's `tenant_id`, `organization_id`, and `user_id`.
- Kept the previously added `async_jobs.provider_id` executability closure intact and aligned the bootstrap profile test fixture so the synthetic local-demo job now uses the same owner tuple as account `7001`.

## Why This Matters

- `async_jobs` are operational evidence. Once a job binds an account, that record is part of the commercial accounting chain rather than loose observability data.
- Allowing a job to point at an existing account owned by another tenant, org, or user would seed commercially invalid execution history into fresh environments.
- Rejecting that state at bootstrap preserves idempotent initialization without letting cross-owner dirty data leak into dev or production deployments.

## Data Impact

- No repository `/data` seed pack required changes for this step.
- The only drift exposed by the new invariant was in the synthetic bootstrap profile fixture inside `crates/sdkwork-api-app-runtime/src/tests.rs`, where `job-local-demo-growth-brief` still used the old `user_id`.
- A targeted ownership audit across repository seed packs found no cross-file account ownership mismatches for:
  - `data/jobs`
  - `data/request-metering`
  - `data/request-settlements`
  - `data/account-benefit-lots`
  - `data/account-holds`
  - `data/account-ledger`

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_async_job_with_missing_account -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_loads_bootstrap_profile_data_pack -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_loads_bootstrap_profile_update_packs_in_order -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_applies_bootstrap_profile_data_idempotently -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`

## Follow-Up

- If bootstrap later gains a stable linkage between async jobs and metered request facts, `async_jobs.request_id` can be tightened with the same commercial ownership guarantees.
- If numeric async-job ownership later maps cleanly to workspace string tenants, provider executability can be tightened from global closure to tenant-accessible closure.
