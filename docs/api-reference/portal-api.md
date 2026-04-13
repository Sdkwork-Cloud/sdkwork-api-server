# Portal API

The portal service exposes the self-service user boundary under `/portal/*`.

## Base URL and Auth

- default local base URL: `http://127.0.0.1:8082/portal`
- health: `GET /portal/health`
- auth boundary: portal JWT, independent from admin JWT

Minimal registration example:

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email":"portal@example.com",
    "password":"PortalPass123!",
    "display_name":"Portal User"
  }'
```

Default local demo login:

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"portal@sdkwork.local",
    "password":"ChangeMe123!"
  }'
```

Password rotation:

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/change-password \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <portal-jwt>" \
  -d '{
    "current_password":"ChangeMe123!",
    "new_password":"PortalPassword456!"
  }'
```

## Route Families

| Family | Routes | Purpose |
|---|---|---|
| health | `GET /portal/health` | liveness |
| auth | `POST /portal/auth/register`, `POST /portal/auth/login`, `GET /portal/auth/me`, `POST /portal/auth/change-password` | end-user registration, session lifecycle, and password rotation |
| workspace | `GET /portal/workspace` | inspect the caller-owned default workspace |
| dashboard | `GET /portal/dashboard` | workspace identity plus a combined usage and billing snapshot for the active project |
| usage | `GET /portal/usage/records`, `GET /portal/usage/summary` | recent requests, token-unit history, and aggregate request counts |
| marketing | `POST /portal/marketing/coupon-validations`, `POST /portal/marketing/coupon-reservations`, `POST /portal/marketing/coupon-redemptions/confirm`, `POST /portal/marketing/coupon-redemptions/rollback`, `GET /portal/marketing/my-coupons`, `GET /portal/marketing/reward-history`, `GET /portal/marketing/redemptions`, `GET /portal/marketing/codes` | coupon eligibility checks, reservation and redemption lifecycle, and caller-owned reward history |
| billing | `GET /portal/billing/summary`, `GET /portal/billing/ledger`, `GET /portal/billing/events`, `GET /portal/billing/events/summary`, `GET /portal/billing/account`, `GET /portal/billing/account/balance`, `GET /portal/billing/account/benefit-lots`, `GET /portal/billing/account/holds`, `GET /portal/billing/account/request-settlements`, `GET /portal/billing/pricing-plans`, `GET /portal/billing/pricing-rates` | quota posture, workspace-scoped Billing 2.0 event inspection, and tenant-facing canonical commercial account visibility |
| API keys | `GET /portal/api-keys`, `POST /portal/api-keys`, `POST /portal/api-keys/{hashed_key}/status`, `DELETE /portal/api-keys/{hashed_key}` | self-service gateway API key lifecycle inside the caller-owned workspace |
| API key groups | `GET /portal/api-key-groups`, `POST /portal/api-key-groups`, `PATCH /portal/api-key-groups/{group_id}`, `POST /portal/api-key-groups/{group_id}/status`, `DELETE /portal/api-key-groups/{group_id}` | self-service API key group lifecycle scoped to the authenticated workspace |
| routing | `GET /portal/routing/summary`, `GET /portal/routing/profiles`, `POST /portal/routing/profiles`, `GET /portal/routing/preferences`, `POST /portal/routing/preferences`, `GET /portal/routing/snapshots`, `POST /portal/routing/preview`, `GET /portal/routing/decision-logs` | workspace-scoped routing posture, reusable profile discovery and save-from-posture flows for group binding, compiled route evidence, preview, and decision evidence |

## Typical User Journey

1. register a portal account
2. log in and receive a portal JWT
3. inspect the default tenant and project workspace
4. open the dashboard snapshot for recent requests, token units, and quota posture
5. review usage and billing detail views
6. inspect the canonical workspace commercial account, benefit lots, active holds, and settled requests
7. inspect workspace billing events and event summaries for group, capability, and accounting-mode visibility
8. validate, reserve, confirm, and if necessary roll back coupon redemptions with replay-safe idempotency controls
9. review workspace routing posture and save the current posture as a reusable routing profile when a group should inherit a stable route plan
10. create one or more API key groups for stable policy attachment
11. issue a gateway API key, optionally binding it to a group with `api_key_group_id`
12. use that key against the gateway `/v1/*` surface

## Browser App

The portal browser experience is a dedicated app:

- `http://127.0.0.1:5174/`

## Related Docs

- product flow:
  - [Public Portal](/getting-started/public-portal)
- operator control plane:
  - [Admin API](/api-reference/admin-api)

## API Key Group Notes

- portal API key group routes are workspace-scoped and never expose groups from another tenant or project
- `slug` is optional on create or update; when omitted it is derived from `name`
- `default_routing_profile_id` is optional on create or update; when present it must resolve to an active routing profile inside the caller workspace
- `GET /portal/routing/profiles` returns only routing profiles inside the authenticated workspace tenant and project
- `POST /portal/routing/profiles` creates a routing profile inside the authenticated workspace tenant and project
- `GET /portal/routing/snapshots` returns only compiled routing snapshots inside the authenticated workspace tenant and project
- `POST /portal/routing/preview` and `GET /portal/routing/decision-logs` surface routing evidence fields when available, including:
  - `compiled_routing_snapshot_id`
  - `fallback_reason`
- portal routing profile creation does not accept caller-supplied `tenant_id`, `project_id`, or `profile_id`; the portal boundary derives those fields from the authenticated workspace and generates a profile id server-side
- routing profile discovery intentionally keeps inactive workspace profiles visible so existing group bindings remain editable and auditable
- compiled routing snapshot discovery is read-only in portal and is intended to explain the effective route state without exposing control-plane mutation
- `default_accounting_mode` is optional on create or update; when present it is normalized to one of:
  - `platform_credit`
  - `byok`
  - `passthrough`
- gateway billing inference for workspace-issued keys now reads `default_accounting_mode` from the bound API key group and otherwise falls back to `platform_credit`
- `POST /portal/api-keys` accepts optional `api_key_group_id`
- key creation rejects groups outside the caller workspace, groups with another environment, and inactive groups

## Billing Event Notes

- `GET /portal/billing/events` returns only billing events inside the authenticated workspace tenant and project
- `GET /portal/billing/events/summary` aggregates only workspace-visible billing events by:
  - project
  - API key group
  - capability
  - accounting mode
- event summaries expose multimodal dimensions already captured by the gateway event ledger, including:
  - token totals
  - `image_count`
  - `audio_seconds`
  - `video_seconds`
  - `music_seconds`

## Marketing Coupon Notes

- `POST /portal/marketing/coupon-validations` and `POST /portal/marketing/coupon-reservations` require `target_kind` so the portal boundary can enforce the coupon template `eligible_target_kinds` restriction against the actual redemption or checkout flow
- `POST /portal/marketing/coupon-reservations`, `POST /portal/marketing/coupon-redemptions/confirm`, and `POST /portal/marketing/coupon-redemptions/rollback` accept an optional JSON `idempotency_key`
- the same three mutation routes also accept the standard `Idempotency-Key` header
- `POST /portal/marketing/coupon-validations`, `POST /portal/marketing/coupon-reservations`, `POST /portal/marketing/coupon-redemptions/confirm`, and `POST /portal/marketing/coupon-redemptions/rollback` are now subject to workspace project coupon rate-limit policies and return `429` when the configured validation, reserve, confirm, or rollback budget is exhausted
- portal coupon throttling evaluates policies against the authenticated workspace project together with the resolved coupon subject bucket such as `project:{project_id}` or `user:{user_id}`, which prevents one caller subject from consuming another subject's coupon quota window
- when both the JSON `idempotency_key` and the `Idempotency-Key` header are supplied, they must normalize to the same value or the request is rejected with `400`
- replaying the same mutation for the same caller subject and the same idempotency key returns the existing reservation, redemption, or rollback instead of creating a duplicate record
- coupon reservation idempotency now includes `target_kind` in the reservation fingerprint, which prevents the same idempotency key from being replayed across incompatible redemption targets
- reusing the same idempotency key for a different coupon mutation payload returns `409`, which protects the reward and checkout ledger from ambiguous retries

## Canonical Commercial Account Notes

- `GET /portal/billing/account` returns the authenticated workspace primary commercial account with summarized available, held, consumed, and granted balance posture
- `GET /portal/billing/account/balance` returns the computed active balance snapshot for the same workspace account
- `GET /portal/billing/account/benefit-lots`, `GET /portal/billing/account/holds`, and `GET /portal/billing/account/request-settlements` are workspace-scoped and never leak another tenant or project account
- `GET /portal/billing/pricing-plans` and `GET /portal/billing/pricing-rates` expose only pricing records attached to the authenticated workspace commercial scope
- portal pricing reads also synchronize due `planned` versions before filtering the workspace scope, so tenant-facing billing posture naturally reflects staged plans once their effective window opens
- portal commercial account routes return `404` when the workspace commercial account has not been provisioned yet, which lets the frontend distinguish bootstrap gaps from authentication failures
