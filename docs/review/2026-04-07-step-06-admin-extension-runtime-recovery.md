# 2026-04-07 Step 06 Admin Extension Runtime Recovery Review

## Scope

This review slice covered the final failing admin extension-runtime verification points that remained after the earlier Step 06 Rust recovery work.

Execution boundary:

- fix concrete verification blockers only
- tighten repository hygiene only where Git noise came from local build outputs
- do not expand into warning cleanup, product-surface changes, or Step 06 closure claims

## Decision Ledger

- Date: `2026-04-07`
- Version: `v0.1.6`
- Wave / Step: `B / 06`
- Primary mode: `verification-solidification`
- Previous mode: `blocker-clearing`
- Strategy switch: yes

### Candidate Actions

1. Fix the remaining admin extension-runtime failures and pre-commit repository hygiene issues before committing `main`.
   - `Priority Score: 106`
   - `S1` current-step closure push: `5 x 5 = 25`
   - `S2` Step 06 capability / `8.3` / `8.6` push: `4 x 5 = 20`
   - `S3` verification and release-gate push: `5 x 4 = 20`
   - `S4` blocker removal value: `4 x 4 = 16`
   - `S5` commercial delivery push: `2 x 3 = 6`
   - `S6` dual-runtime consistency value: `3 x 3 = 9`
   - `S7` immediate verifiability: `5 x 2 = 10`
   - `P1` churn / rework risk: `0 x -3 = 0`

2. Commit the current dirty `main` state immediately and accept known failing proof points.
   - `Priority Score: 47`
   - rejected because it would codify known failures into the mainline history

3. Prioritize warning cleanup over failing-proof recovery.
   - `Priority Score: 58`
   - rejected because warnings were lower severity than the active red tests

### Chosen Action

Action 1 was selected because it directly removed known failing Step 06 proofs and produced a safer mainline commit candidate with minimal write-surface expansion.

## Root Cause Summary

### 1. Native Dynamic Runtime Fixture Missing

The admin test binary did not declare `sdkwork-api-ext-provider-native-mock` as a dev dependency and used a duplicated string constant for `FIXTURE_EXTENSION_ID`.

Result:

- the admin test executable did not build or expose the native mock dynamic library under its `deps` directory
- `native_dynamic_fixture_library_path()` failed at runtime because no matching fixture library existed

### 2. Extension Discovery Race

The extension package discovery tests mutate shared `SDKWORK_EXTENSION_*` process environment variables but were not serialized.

Result:

- concurrent test execution could observe a different discovery policy than the test expected
- `list_discovered_extension_packages_from_admin_api` intermittently saw `0` packages instead of `1`

### 3. Repository Hygiene Noise

The repository ignored the canonical `/target/` directory but not the many local `target-*` verification roots or the temporary `tmp` and `bin/.sdkwork-target` roots created during repeated local validation.

Result:

- Git status remained noisy even when only source and documentation changes were intended for commit

## Implemented Fixes

- added `sdkwork-api-ext-provider-native-mock` to `crates/sdkwork-api-interface-admin/Cargo.toml` dev dependencies
- imported `FIXTURE_EXTENSION_ID` from the native mock crate in admin test support
- added `#[serial(extension_env)]` to the environment-sensitive extension package discovery tests
- extended `.gitignore` to ignore local `target-*`, `tmp`, and `bin/.sdkwork-target` artifact roots

## Files Touched In This Slice

- `.gitignore`
- `crates/sdkwork-api-interface-admin/Cargo.toml`
- `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes/support.rs`
- `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes/extensions_runtime.rs`

## Verification Evidence

### Green

- `CARGO_TARGET_DIR=target-precommit-step06-admin-sqlite cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -- --nocapture`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-admin/tests/admin-commercial-workbench.test.mjs apps/sdkwork-router-admin/tests/admin-i18n-coverage.test.mjs`

### Observed Constraint

- `node --test ...` under the default isolated test runner mode still failed with `spawn EPERM` inside the sandbox
- the non-isolated runner mode is required in the current environment to obtain frontend verification evidence

## Current Assessment

### Closed In This Slice

- admin extension package discovery verification is stable again
- admin native dynamic runtime status verification is stable again
- admin extension runtime reload verification is stable again
- Git noise from local verification target directories is now ignored at the repository level

### Still Open

- Step 06 overall capability closure is still incomplete
- release alignment for the broader Wave `B` accumulation still needs a final commit-level update
- mainline commit / push still depends on the final repository snapshot action

## Maturity Delta

- `stateful standalone` fact maturity: `L3 -> L3+`
- `stateless runtime` fact maturity: unchanged at `L2`

## Next Slice Recommendation

1. create the mainline snapshot commit for the full accumulated repository state
2. record the final committed release ledger after push status is known
3. continue Step 06 from verification-solidification into wider control-plane and commercialization closure
