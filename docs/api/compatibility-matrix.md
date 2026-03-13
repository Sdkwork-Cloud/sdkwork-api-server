# Compatibility Matrix

## Current Implemented Endpoints

| Endpoint | Status | Notes |
|---|---|---|
| `/v1/models` | Implemented | Catalog-backed through SQLite when the stateful gateway is used |
| `/v1/chat/completions` | Implemented | Stateful mode supports OpenAI-compatible upstream relay for non-stream and `text/event-stream`; falls back to stub output when provider execution is unavailable |
| `/v1/responses` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local response object fallback |
| `/v1/embeddings` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local embeddings fallback |

## Current Implemented Admin APIs

| Endpoint Family | Status | Notes |
|---|---|---|
| `/admin/auth/login` | Implemented | Issues and verifies SDKWork-style token payloads |
| `/admin/tenants` | Implemented | SQLite-backed list and create |
| `/admin/projects` | Implemented | SQLite-backed list and create |
| `/admin/api-keys` | Implemented | SQLite-backed issuance and list |
| `/admin/channels` | Implemented | SQLite-backed list and create |
| `/admin/providers` | Implemented | SQLite-backed list and create with `adapter_kind` and `base_url` execution config |
| `/admin/models` | Implemented | SQLite-backed list and create |
| `/admin/credentials` | Implemented | SQLite-backed encrypted secret storage and credential reference listing |
| `/admin/routing/simulations` | Implemented | Catalog-backed route simulation |
| `/admin/usage/records` | Implemented | Lists gateway-recorded usage events |
| `/admin/billing/ledger` | Implemented | Lists booked cost entries |

## Defined Contract Families

| API Family | Contract Status | Execution Status |
|---|---|---|
| Models | Defined | Implemented |
| Chat Completions | Defined | Implemented |
| Completions | Defined | Contract only |
| Responses | Defined | Implemented |
| Embeddings | Defined | Implemented |
| Streaming | Defined | Implemented |
| Files | Defined | Contract only |
| Uploads | Defined | Contract only |
| Audio | Defined | Contract only |
| Images | Defined | Contract only |
| Moderations | Defined | Contract only |
| Fine Tuning | Defined | Contract only |
| Realtime | Defined | Contract only |
| Assistants | Defined | Contract only |
| Vector Stores | Defined | Contract only |
| Batches | Defined | Contract only |
| Videos | Defined | Contract only |
| Webhooks | Defined | Contract only |
| Evals | Defined | Contract only |

## Runtime Behavior Notes

| Capability | Current Behavior |
|---|---|
| Upstream proxying | Partially implemented; stateful gateway relays OpenAI-compatible chat, responses, embeddings, and chat SSE when provider, model, and credential records are configured |
| Model discovery | Driven by the local catalog, not upstream auto-sync |
| Routing | Deterministic candidate selection from catalog models |
| Provider dispatch | Executed through `sdkwork-api-provider-core` registry abstractions with `adapter_kind` plus `base_url` resolution; `openai`, `openrouter`, and `ollama` are currently registered |
| Credential handling | Upstream secrets are encrypted at rest and resolved with `credential_master_key` during execution; active persistence backends are `database_encrypted`, `local_encrypted_file`, and `os_keyring` |
| Usage tracking | Persisted through admin SQLite store |
| Billing | Ledger entries booked from gateway-side request hooks |

## Storage Support

| Driver | Status |
|---|---|
| SQLite | Active implementation with control-plane and telemetry persistence |
| PostgreSQL | Active implementation with shared admin store parity and standalone service startup support |
| MySQL | Boundary crate implemented |
| libsql | Boundary crate implemented |
