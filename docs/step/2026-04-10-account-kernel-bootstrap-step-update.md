# 2026-04-10 Account Kernel Bootstrap Step Update

## Completed

### Step 1 - `/data` bootstrap framework extended into the commercial account kernel

- kept the existing repository bootstrap contract:
  - `data/profiles/*.json`
  - `data/updates/*.json`
  - `data/<domain>/*.json`
- added first-class account-kernel domains to the typed bootstrap pack:
  - `accounts`
  - `account-benefit-lots`
  - `account-holds`
  - `account-ledger`
  - `request-metering`
  - `request-settlements`
  - `account-reconciliation`
- kept strong bundle cohesion only where the records are lifecycle-coupled:
  - `account-holds` = holds + allocations
  - `account-ledger` = entries + allocations
  - `request-metering` = facts + metrics

### Step 2 - Idempotent update-pack evolution preserved

- profile manifests merge ordered update manifests before loading domain JSON
- merged records collapse with stable-key last-wins semantics
- repeated startup or deployment does not duplicate account, hold, ledger, metering, settlement, or reconciliation rows
- data evolution now happens through additive update packs instead of mutating baseline files in place

### Step 3 - Cross-domain validation tightened

- account ownership and timestamp consistency are validated
- hold/allocation and ledger/allocation relationships are validated
- request metering validates references to:
  - accounts
  - channels
  - providers
  - provider-supported models
  - api keys
  - pricing plans
- request settlement validates hold/account/request consistency
- account reconciliation validates project/order linkage

### Step 4 - Bootstrap apply pipeline split by responsibility

- catalog, identity, routing, commerce, jobs, and runtime-governance records continue through `AdminStore`
- account kernel records flow through `AccountKernelStore`
- pricing plan/rate writes flow through `CommercialBillingAdminKernel`

This keeps the initialization framework high-cohesion, low-coupling, and extensible while preserving one unified profile-driven bootstrap experience.

### Step 5 - Production and developer data packs expanded

- production profile now includes `2026-04-account-kernel-commercial-foundation`
- developer profile now overlays `2026-04-dev-account-kernel-expansion`
- fresh installs now start with inspectable commercial state instead of only catalog metadata:
  - payable accounts
  - active benefit lots
  - request holds
  - ledger history
  - metering facts and metrics
  - settlements
  - reconciliation state

### Step 6 - Channel / provider / model / price semantics held stable

- `channel` remains the canonical inventor/vendor surface
- `provider` remains the official, proxy, or local execution endpoint
- `provider-model` remains the explicit coverage declaration for proxy/local subsets
- `route config` remains provider-centric
- `model-price` remains the official/proxy/local catalog price surface
- `pricing-plan` and `pricing-rate` remain the internal commercial billing contract

This keeps official pricing, proxy pricing, routing validity, and settlement accounting explainable without collapsing them into one ambiguous table.

## Verified

- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

Both commands passed in the current workspace.

## Next

- keep upstream catalog and billing changes flowing through new `data/updates/*.json` packs
- continue admin-side provider coverage management around `provider-model` subsets without assuming full-channel proxy coverage
- keep request metering and settlement samples aligned to future pricing-plan or route-matrix updates so the repository bootstrap remains production-credible
