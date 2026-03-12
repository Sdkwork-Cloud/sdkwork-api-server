# Compatibility Matrix

## Current Implemented Endpoints

| Endpoint | Status | Notes |
|---|---|---|
| `/v1/models` | Implemented | Catalog-backed through SQLite when the stateful gateway is used |
| `/v1/chat/completions` | Implemented | Minimal JSON response, SSE path, usage recording, and billing booking |
| `/v1/responses` | Implemented | Minimal response object with usage recording and billing booking |
| `/v1/embeddings` | Implemented | Minimal embeddings response with usage recording and billing booking |

## Current Implemented Admin APIs

| Endpoint Family | Status | Notes |
|---|---|---|
| `/admin/auth/login` | Implemented | Issues and verifies SDKWork-style token payloads |
| `/admin/tenants` | Implemented | SQLite-backed list and create |
| `/admin/projects` | Implemented | SQLite-backed list and create |
| `/admin/api-keys` | Implemented | SQLite-backed issuance and list |
| `/admin/channels` | Implemented | SQLite-backed list and create |
| `/admin/providers` | Implemented | SQLite-backed list and create |
| `/admin/models` | Implemented | SQLite-backed list and create |
| `/admin/routing/simulations` | Implemented | Catalog-backed route simulation |
| `/admin/usage/records` | Implemented | Lists gateway-recorded usage events |
| `/admin/billing/ledger` | Implemented | Lists booked cost entries |

## Defined Contract Families

| API Family | Contract Status | Execution Status |
|---|---|---|
| Models | Defined | Implemented |
| Chat Completions | Defined | Implemented |
| Responses | Defined | Implemented |
| Embeddings | Defined | Implemented |
| Streaming | Defined | Implemented |
| Files | Planned | Not implemented |
| Uploads | Planned | Not implemented |
| Audio | Planned | Not implemented |
| Images | Planned | Not implemented |
| Moderations | Planned | Not implemented |
| Realtime | Planned | Not implemented |
| Assistants | Planned | Not implemented |
| Vector Stores | Planned | Not implemented |
| Webhooks | Planned | Not implemented |
| Evals | Planned | Not implemented |

## Runtime Behavior Notes

| Capability | Current Behavior |
|---|---|
| Upstream proxying | Planned next; current gateway emits stubbed responses |
| Model discovery | Driven by the local catalog, not upstream auto-sync |
| Routing | Deterministic candidate selection from catalog models |
| Usage tracking | Persisted through admin SQLite store |
| Billing | Ledger entries booked from gateway-side request hooks |

## Storage Support

| Driver | Status |
|---|---|
| SQLite | Active implementation with control-plane and telemetry persistence |
| PostgreSQL | Boundary crate implemented |
| MySQL | Boundary crate implemented |
| libsql | Boundary crate implemented |
