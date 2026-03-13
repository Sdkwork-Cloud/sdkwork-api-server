# Compatibility Matrix

## Current Implemented Endpoints

| Endpoint | Status | Notes |
|---|---|---|
| `/v1/models` | Implemented | Catalog-backed through SQLite when the stateful gateway is used |
| `/v1/chat/completions` | Implemented | Stateful mode supports OpenAI-compatible upstream relay for non-stream and `text/event-stream`; falls back to stub output when provider execution is unavailable |
| `/v1/completions` | Implemented | Stateful mode supports OpenAI-compatible upstream relay for legacy text completions; otherwise emits local completion fallback |
| `/v1/responses` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local response object fallback |
| `/v1/embeddings` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local embeddings fallback |
| `/v1/files` | Implemented | Stateful mode supports OpenAI-compatible upstream multipart relay; otherwise emits local file fallback |
| `/v1/moderations` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local unflagged moderation fallback |
| `/v1/images/generations` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local base64 image fallback |
| `/v1/audio/transcriptions` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local transcription fallback |
| `/v1/audio/translations` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local translation fallback |
| `/v1/audio/speech` | Implemented | Stateful mode supports OpenAI-compatible upstream binary or event-stream relay; otherwise emits local audio fallback |
| `/v1/uploads` | Implemented | Stateful mode supports OpenAI-compatible upstream JSON relay for upload creation; otherwise emits local upload fallback |
| `/v1/uploads/{upload_id}/parts` | Implemented | Stateful mode supports OpenAI-compatible upstream multipart relay for upload parts; otherwise emits local upload-part fallback |
| `/v1/uploads/{upload_id}/complete` | Implemented | Stateful mode supports OpenAI-compatible upstream relay for upload completion; otherwise emits local completion fallback |
| `/v1/uploads/{upload_id}/cancel` | Implemented | Stateful mode supports OpenAI-compatible upstream relay for upload cancellation; otherwise emits local cancelled-upload fallback |
| `/v1/fine_tuning/jobs` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local fine-tuning job fallback |
| `/v1/assistants` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local assistant fallback |
| `/v1/realtime/sessions` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local realtime session fallback |
| `/v1/evals` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local eval fallback |
| `/v1/batches` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local batch fallback |
| `/v1/vector_stores` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local vector store fallback |

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
| Completions | Defined | Implemented |
| Responses | Defined | Implemented |
| Embeddings | Defined | Implemented |
| Moderations | Defined | Implemented |
| Images | Defined | Implemented |
| Streaming | Defined | Implemented |
| Files | Defined | Implemented |
| Uploads | Defined | Implemented |
| Audio | Defined | Partially implemented (`/v1/audio/transcriptions`, `/v1/audio/translations`, `/v1/audio/speech`) |
| Fine Tuning | Defined | Implemented |
| Realtime | Defined | Implemented |
| Assistants | Defined | Implemented |
| Vector Stores | Defined | Implemented |
| Batches | Defined | Implemented |
| Videos | Defined | Contract only |
| Webhooks | Defined | Contract only |
| Evals | Defined | Implemented |

## Runtime Behavior Notes

| Capability | Current Behavior |
|---|---|
| Upstream proxying | Partially implemented; stateful gateway relays OpenAI-compatible chat, chat SSE, completions, responses, embeddings, files multipart uploads, upload create/part/complete/cancel, moderations, image generations, audio transcriptions, audio translations, audio speech binary passthrough, fine-tuning jobs, assistants, realtime sessions, evals, batches, and vector stores when provider and credential records are configured |
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
