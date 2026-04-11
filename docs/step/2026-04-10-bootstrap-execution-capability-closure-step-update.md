# 2026-04-10 Bootstrap Execution Capability Closure Step Update

## What Changed

- Hardened bootstrap validation so `async_jobs.capability_code` must be supported by the resolved provider model metadata for the declared `provider_id + model_code`.
- Hardened bootstrap validation so `request_meter_facts.capability_code` must be supported by the resolved `channel_code + model_code` channel model.
- Added regression tests for:
  - async jobs that declare a capability unsupported by the selected provider model
  - request metering facts that declare a capability unsupported by the selected channel model

## Why This Matters

- Commercial bootstrap data should reject execution evidence that names a real model but assigns it an impossible capability.
- Without this closure, seeded environments could load:
  - embeddings traffic recorded against chat-only models
  - responses traffic recorded against embedding-only models
- That kind of data drift is structurally valid enough to survive loose bootstrap checks, but it is commercially unsafe because pricing, routing, analytics, and admin diagnostics all depend on capability semantics.

## Data Impact

- No repository `/data` seed pack required changes.
- Audits confirmed:
  - `data/jobs/*.json` has no provider-model capability drift
  - `data/request-metering/*.json` has no channel-model capability drift
- One unrelated synthetic test fixture override had to restore `openrouter::gpt-4.1` channel-model metadata so older tests still satisfy the stronger metering closure.

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_async_job_with_capability_not_supported_by_model -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_request_meter_fact_with_capability_not_supported_by_channel_model -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- The next capability-closure candidate is provider-specific metering support: `request_meter_facts.provider_code` could be tightened from channel-model capability coverage to provider-model capability coverage where proxy providers expose narrower capability subsets than the canonical channel model.
