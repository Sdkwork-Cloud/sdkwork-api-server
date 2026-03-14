# Public Portal

The public portal exposes a self-service user experience that is intentionally separate from the operator-only admin control plane.

## Portal Routes

- `POST /portal/auth/register`
- `POST /portal/auth/login`
- `GET /portal/auth/me`
- `GET /portal/workspace`
- `GET /portal/api-keys`
- `POST /portal/api-keys`

## Browser Routes

- `#/portal/register`
- `#/portal/login`
- `#/portal/dashboard`

## Default Portal Flow

1. Open `http://127.0.0.1:5173/#/portal/register`
2. Create a portal account
3. Log in or land on the dashboard
4. Inspect the default workspace
5. Create a gateway API key
6. Copy the plaintext key immediately
7. Use that key against the gateway

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
- self-service gateway API key issuance

It intentionally does not yet include:

- invitations
- multi-workspace membership
- password reset email
- OAuth or SSO
