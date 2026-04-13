# Checkout Artifact Recovery Design

## Scope

This tranche hardens the boundary between portal commerce orders and canonical payment preparation:

- payable orders must tolerate partial loss of canonical checkout artifacts
- order-center reads should be able to self-heal missing payment order / attempt / session state
- recovery must not downgrade already-advanced payment state produced by verified callbacks

Out of scope:

- provider-native checkout session provisioning for Stripe, WeChat Pay, or Alipay
- background reconciliation jobs
- admin-visible recovery dashboards

## Problem

The portal currently creates canonical payment artifacts as a follow-up step after commerce order creation. The insert path is deterministic and idempotent, but the read side still assumes those artifacts are fully present.

That leaves a commercial gap:

- if checkout preparation partially fails after order creation, the order may exist without complete payment artifacts
- `GET /portal/commerce/order-center` does not currently repair that state
- naive artifact re-creation can accidentally overwrite more advanced payment state, for example when a verified callback has already moved a payment order to `captured` while the commerce order is still temporarily `pending_payment`

The system needs a recovery-safe way to rehydrate checkout artifacts without downgrading trusted payment progress.

## Design

### 1. Make checkout ensure logic recovery-safe

`ensure_commerce_payment_checkout(...)` should stop behaving like a blind overwrite. Instead:

- create deterministic desired records for payment order / attempt / session
- load any existing canonical records
- preserve advanced existing statuses instead of downgrading to earlier defaults
- only write records when they are missing or actually need forward reconciliation

Recovery rules:

- existing `captured`, `failed`, `expired`, `canceled`, and in-flight `processing` payment states win over an earlier desired state such as `awaiting_customer`
- desired terminal states derived from the commerce order can still advance an older `awaiting_customer` record
- missing child attempt/session records should be recreated using the recovered parent state so they align with the current payment lifecycle

### 2. Recover artifacts from the portal order center

Before building the portal order-center projection:

- load project-scoped commerce orders
- best-effort run checkout recovery for payable orders using the order owner recorded on each commerce order
- reload the order-center projection after recovery

This turns order-center reads into a safe recovery surface for the most important business view without requiring a separate repair job.

### 3. Preserve availability during recovery

Order-center recovery should be best-effort:

- if a specific order cannot be repaired, do not corrupt or downgrade existing payment data
- still return the best currently persisted order-center projection instead of failing the entire route solely because recovery was unavailable

This favors operator and customer visibility during degraded conditions while keeping the write path conservative.

## Testing strategy

Add or update regression coverage for:

- payment ensure logic preserving advanced payment state while recreating missing child records
- portal order-center reads recreating missing payment artifacts for payable orders
- existing portal / payment / admin regressions staying green after the recovery logic changes
