# Public Portal

The public portal exposes a self-service user experience that is intentionally separate from the operator-only admin control plane.

It is the default end-user boundary for account creation, dashboard and usage review, billing posture, and gateway API key issuance.

## Portal Routes

- `POST /portal/auth/register`
- `POST /portal/auth/login`
- `GET /portal/auth/me`
- `POST /portal/auth/change-password`
- `GET /portal/workspace`
- `GET /portal/dashboard`
- `GET /portal/commerce/catalog`
- `GET /portal/usage/records`
- `GET /portal/usage/summary`
- `GET /portal/billing/summary`
- `GET /portal/billing/ledger`
- `GET /portal/api-keys`
- `POST /portal/api-keys`

## Browser App

- `http://127.0.0.1:5174/`

## Default Portal Flow

1. Open `http://127.0.0.1:5174/`
2. Create a portal account, or, only in explicit development mode, log in with the seeded local demo account
3. Log in or land on the dashboard
4. Inspect workspace identity, recent requests, token-unit usage, and billing posture
5. Review coupon redemption, recharge, and subscription entry points inside the portal
6. Create a gateway API key
7. Copy the plaintext key immediately
8. Use that key against the gateway

Local demo account, available only when `SDKWORK_ALLOW_LOCAL_DEV_BOOTSTRAP=true`:

- email: `portal@sdkwork.local`
- password: `ChangeMe123!`

Example:

```bash
curl http://127.0.0.1:8080/v1/models \
  -H "Authorization: Bearer skw_live_your_key_here"
```

## Security Boundary

Portal authentication is separate from admin authentication:

- different route namespace
- different JWT boundary
- portal users only see their own default tenant and project scope

## Current Scope

The current portal batch intentionally supports:

- registration
- login
- workspace inspection
- dashboard snapshot with recent requests
- usage workbench and per-call token-unit visibility
- billing summary and ledger reads, with recharge workspaces surfacing canonical account balance as the effective available amount
- backend-readable subscription, recharge, and coupon catalog plus frontend entry points
- self-service gateway API key issuance

It intentionally does not yet include:

- invitations
- multi-workspace membership
- password reset email
- OAuth or SSO
- live checkout and payment settlement

## Commerce Safety Notes

- portal JWT users cannot directly post paid settlement events anymore
- paid orders should finish through the server-side payment callback seam, not through a portal browser action
- `SDKWORK_PORTAL_ALLOW_MANUAL_SETTLEMENT=true` exists only for explicit lab or operator validation flows and should stay disabled in commercial environments
- payment providers should call `POST /portal/internal/commerce/orders/{order_id}/payment-events` with `x-sdkwork-payment-callback-secret`
- after a recharge payment settles, `/portal/billing/summary` reports the effective available balance from the canonical recharge account and preserves quota compatibility values separately through `quota_remaining_units`

## Related Docs

- local startup:
  - [Source Development](/getting-started/source-development)
- service boundaries:
  - [Portal API Reference](/api-reference/portal-api)
- control-plane distinction:
  - [Admin API Reference](/api-reference/admin-api)
