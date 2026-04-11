# 2026-04-11 Bootstrap Async Job Format Semantics Closure Step Update

## What Changed

- Hardened bootstrap validation for async-job semantic formatting, beyond the earlier parent/child evidence checks.
- Added attempt/provider alignment:
  - `async_job_attempts.runtime_kind` must match the normalized parent `async_jobs.provider_id`
- Added asset format alignment for known seeded asset kinds:
  - `asset_kind = json` => `mime_type = application/json`, storage leaf ends with `.json`
  - `asset_kind = markdown` => `mime_type = text/markdown`, storage leaf ends with `.md`
  - `download_url` leaf, when present, must equal the `storage_key` leaf
- Added completed-callback formatting guarantees:
  - parent jobs with `completed_at_ms` must use `event_type = job.completed`
  - `job.completed` callbacks must use `dedupe_key` ending with `:completed`

## Why This Matters

- The bootstrap pack is supposed to be install-ready commercial data, not loosely coherent demo payloads.
- The previous round proved lifecycle and payload evidence integrity, but still allowed semantic drift such as:
  - an `openrouter` job attempt being tagged as `gemini`
  - a `markdown` or `json` asset declaring the wrong MIME or file extension
  - a completed callback being labeled as progress or using a non-terminal dedupe suffix
- Those are the kinds of silent inconsistencies that make admin audit trails, seeded job assets, and callback replay behavior untrustworthy.

## Repository Audit

- Re-audited real merged bootstrap job data from `data/jobs/*.json` plus ordered profile updates.
- Observed seeded runtime kinds across current packs:
  - `openai`, `openrouter`, `anthropic`, `gemini`, `ollama`, `siliconflow`, `minimax`, `ernie`
- Observed seeded asset kinds across current packs:
  - `json`
  - `markdown`
- Observed result:
  - no attempt `runtime_kind` drift relative to normalized parent provider ids
  - no asset MIME drift for seeded `json` / `markdown` assets
  - no asset storage extension drift for seeded `json` / `markdown` assets
  - no download leaf drift relative to storage leaf
  - no completed callback event-type drift
  - no completed callback dedupe suffix drift

## Test Refinement

- Corrected two newly added red tests so they mutate the fixture into a real mismatch instead of accidentally preserving valid `json` semantics:
  - MIME mismatch test now writes `text/markdown` onto the seeded `json` asset
  - storage extension mismatch test now writes `output.md` onto the seeded `json` asset
- After refinement, all six new tests failed for the intended missing-manifest-check reason before production code was added.

## Data Impact

- No `/data` bootstrap files required modification.
- Existing dev/prod-ready seed data already satisfied the stronger semantic contract.
- This round only promoted existing de facto conventions into explicit bootstrap invariants.

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_async_job_ -- --nocapture`
  - passed: `25 passed; 0 failed`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
  - passed: `186 passed; 0 failed`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`
  - passed: `2 passed; 0 failed`

## Follow-Up

- If later bootstrap packs introduce additional asset kinds, extend the MIME and extension mapping deliberately instead of weakening validation.
- If later product flows persist non-terminal webhook history for completed jobs, make that a first-class callback model change instead of loosening the current terminal-callback contract.
