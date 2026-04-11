# 2026-04-10 Bootstrap Account Reconciliation Order Ownership Integrity Step Update

## What Changed

- Hardened bootstrap validation for `account_commerce_reconciliation_states`.
- A reconciliation state now fails bootstrap unless:
  - its `tenant_id + organization_id` matches the referenced `account_id`
  - its `last_order_created_at_ms` matches the referenced order `created_at_ms`
  - its `last_order_updated_at_ms` matches the referenced order `updated_at_ms`

## Why This Matters

- `account_reconciliation` is the account-kernel projection that portal/admin surfaces use to explain the latest commercial checkpoint for an account and project.
- Previous validation already required:
  - referenced account exists
  - referenced project exists
  - referenced order exists
  - referenced order belongs to the same project
  - local timestamp ordering is monotonic
- That still allowed two subtle integrity gaps:
  - the reconciliation record could claim another workspace's numeric ownership while reusing a valid account id
  - the record could point at a valid latest order id while carrying stale or fabricated order timestamps
- In a commercial bootstrap pack, both would make seeded ledger and settlement projections less trustworthy.

## Repository Audit

- Re-audited merged `prod` and `dev` profile packs across:
  - `data/accounts/*.json`
  - `data/account-reconciliation/*.json`
  - `data/commerce/*.json`
  - additive `data/updates/*.json`
- Audit result:
  - `PROFILE=prod ACCOUNT_RECON_BAD=0`
  - `PROFILE=dev ACCOUNT_RECON_BAD=0`
- This confirmed the repository data already treats reconciliation as an exact projection of the linked account + latest order checkpoint.

## Data Impact

- No `/data` seed files required changes.
- Existing production and development bootstrap packs already satisfy the stronger reconciliation contract.
- This step only makes an existing semantic guarantee explicit and enforceable.

## Test Coverage Added

- reconciliation rejects tenant/organization drift from the linked account
- reconciliation rejects last-order timestamp drift from the linked commerce order

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_account_reconciliation_with_mismatched_account_ownership -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_account_reconciliation_with_mismatched_last_order_timestamps -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`
- Real `/data` audit:
  - `PROFILE=prod ACCOUNT_RECON_BAD=0`
  - `PROFILE=dev ACCOUNT_RECON_BAD=0`

## Follow-Up

- Future reconciliation seeds should continue to treat `last_order_*` as exact order-projection fields, not approximate dashboard hints.
- If reconciliation expands to track more finance review state, keep the projection explicit instead of embedding derived ambiguity into bootstrap data.
