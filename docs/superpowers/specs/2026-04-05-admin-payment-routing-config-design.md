# Admin Payment Routing Configuration Design

## Scope

This tranche makes payment routing configuration operationally manageable from the admin API.

It covers:

- payment gateway account inspection and upsert from admin routes
- payment channel policy inspection and upsert from admin routes
- deterministic filtering and sort order for operator views
- basic validation to prevent obviously broken routing records from entering the system

It does not cover:

- delete endpoints
- audit logs
- bulk import/export
- portal-facing routing management
- advanced routing simulation or health scoring

## Problem

The payment service now supports automatic failover between configured routes, but the
configuration is still effectively internal-only:

- gateway accounts and channel policies can be persisted in storage, but not managed through the
  admin API
- operators cannot inspect active/inactive routing state from the control plane
- failover readiness depends on out-of-band data seeding instead of first-class product surfaces

That keeps the routing foundation below commercial operating standards.

## Recommended Approach

Expose payment routing configuration through the existing admin payment surface with explicit
list and upsert routes:

- `GET /admin/payments/gateway-accounts`
- `POST /admin/payments/gateway-accounts`
- `GET /admin/payments/channel-policies`
- `POST /admin/payments/channel-policies`

The routes should be implemented in the admin interface crate, using the existing storage-backed
records directly. This keeps the slice small, avoids introducing a premature application service,
and matches the current admin payment reconciliation pattern.

## API Design

### Gateway accounts

List query parameters:

- `provider_code` optional exact match
- `status` optional exact match
- `tenant_id` optional exact match
- `organization_id` optional exact match

Sorting:

1. `priority DESC`
2. `updated_at_ms DESC`
3. `gateway_account_id ASC`

Upsert request fields:

- `gateway_account_id`
- `tenant_id`
- `organization_id`
- `provider_code`
- `environment`
- `merchant_id`
- `app_id`
- `status`
- `priority`
- `created_at_ms` optional
- `updated_at_ms` optional

Validation:

- id, environment, merchant id, status must be non-empty
- provider code must be a supported payment provider
- status is constrained to `active` or `inactive`
- priority is accepted as-is and may be negative

Response:

- stored `PaymentGatewayAccountRecord`

### Channel policies

List query parameters:

- `provider_code` optional exact match
- `status` optional exact match
- `tenant_id` optional exact match
- `organization_id` optional exact match
- `scene_code` optional exact match
- `currency_code` optional exact match
- `client_kind` optional exact match

Sorting:

1. `priority DESC`
2. `updated_at_ms DESC`
3. `channel_policy_id ASC`

Upsert request fields:

- `channel_policy_id`
- `tenant_id`
- `organization_id`
- `scene_code`
- `country_code`
- `currency_code`
- `client_kind`
- `provider_code`
- `method_code`
- `priority`
- `status`
- `created_at_ms` optional
- `updated_at_ms` optional

Validation:

- id, method code, status must be non-empty
- provider code must be a supported payment provider
- status is constrained to `active` or `inactive`
- empty scene/country/currency/client fields remain allowed for wildcard matching

Response:

- stored `PaymentChannelPolicyRecord`

## Operational Behavior

- The list endpoints are read-only operator surfaces and return stored records after deterministic
  filtering and sorting.
- The post endpoints behave as idempotent upserts keyed by the record id.
- `updated_at_ms` defaults to current time if omitted.
- `created_at_ms` defaults to `updated_at_ms` when omitted for first write semantics.
- Invalid payloads return `400 Bad Request`.
- Unsupported store backends keep the existing admin error pattern and surface `500` because this
  project currently treats those as unavailable admin capabilities rather than feature-negotiated
  APIs.

## Testing Strategy

Add admin interface coverage for:

1. upserting and listing gateway accounts with filtering and ordering
2. upserting and listing channel policies with filtering and ordering
3. rejecting malformed routing configuration payloads

Re-run the existing admin payment tests to ensure reconciliation and order/refund surfaces remain
stable.
