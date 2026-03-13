# Compatibility Matrix

## Current Implemented Endpoints

| Endpoint | Status | Notes |
|---|---|---|
| `/v1/models` | Implemented | Catalog-backed through SQLite when the stateful gateway is used |
| `/v1/models/{model}` | Implemented | Catalog-backed model retrieval through SQLite when the stateful gateway is used |
| `/v1/chat/completions` | Implemented | Stateful mode supports OpenAI-compatible upstream relay for non-stream and `text/event-stream`; falls back to stub output when provider execution is unavailable |
| `/v1/completions` | Implemented | Stateful mode supports OpenAI-compatible upstream relay for legacy text completions; otherwise emits local completion fallback |
| `/v1/responses` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local response object fallback |
| `/v1/embeddings` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local embeddings fallback |
| `/v1/files` | Implemented | Stateful mode supports OpenAI-compatible upstream multipart create and JSON list relay; otherwise emits local file fallback |
| `/v1/files/{file_id}` | Implemented | Stateful mode supports OpenAI-compatible upstream retrieve and delete relay; otherwise emits local file metadata or deleted response fallback |
| `/v1/files/{file_id}/content` | Implemented | Stateful mode supports OpenAI-compatible upstream binary passthrough; otherwise emits local file-content fallback |
| `/v1/moderations` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local unflagged moderation fallback |
| `/v1/images/generations` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local base64 image fallback |
| `/v1/audio/transcriptions` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local transcription fallback |
| `/v1/audio/translations` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local translation fallback |
| `/v1/audio/speech` | Implemented | Stateful mode supports OpenAI-compatible upstream binary or event-stream relay; otherwise emits local audio fallback |
| `/v1/uploads` | Implemented | Stateful mode supports OpenAI-compatible upstream JSON relay for upload creation; otherwise emits local upload fallback |
| `/v1/uploads/{upload_id}/parts` | Implemented | Stateful mode supports OpenAI-compatible upstream multipart relay for upload parts; otherwise emits local upload-part fallback |
| `/v1/uploads/{upload_id}/complete` | Implemented | Stateful mode supports OpenAI-compatible upstream relay for upload completion; otherwise emits local completion fallback |
| `/v1/uploads/{upload_id}/cancel` | Implemented | Stateful mode supports OpenAI-compatible upstream relay for upload cancellation; otherwise emits local cancelled-upload fallback |
| `/v1/fine_tuning/jobs` | Implemented | Stateful mode supports OpenAI-compatible upstream create and list relay; otherwise emits local fine-tuning job fallback |
| `/v1/fine_tuning/jobs/{fine_tuning_job_id}` | Implemented | Stateful mode supports OpenAI-compatible upstream retrieve relay; otherwise emits local fine-tuning job metadata fallback |
| `/v1/fine_tuning/jobs/{fine_tuning_job_id}/cancel` | Implemented | Stateful mode supports OpenAI-compatible upstream cancel relay; otherwise emits local cancelled fine-tuning job fallback |
| `/v1/assistants` | Implemented | Stateful mode supports OpenAI-compatible upstream create and list relay; otherwise emits local assistant fallback |
| `/v1/assistants/{assistant_id}` | Implemented | Stateful mode supports OpenAI-compatible upstream retrieve, update, and delete relay; otherwise emits local assistant metadata or deleted response fallback |
| `/v1/webhooks` | Implemented | Stateful mode supports OpenAI-compatible upstream create and list relay; otherwise emits local webhook fallback |
| `/v1/webhooks/{webhook_id}` | Implemented | Stateful mode supports OpenAI-compatible upstream retrieve, update, and delete relay; otherwise emits local webhook metadata or deleted response fallback |
| `/v1/realtime/sessions` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local realtime session fallback |
| `/v1/evals` | Implemented | Stateful mode supports OpenAI-compatible upstream relay; otherwise emits local eval fallback |
| `/v1/batches` | Implemented | Stateful mode supports OpenAI-compatible upstream create and list relay; otherwise emits local batch fallback |
| `/v1/batches/{batch_id}` | Implemented | Stateful mode supports OpenAI-compatible upstream retrieve relay; otherwise emits local batch metadata fallback |
| `/v1/batches/{batch_id}/cancel` | Implemented | Stateful mode supports OpenAI-compatible upstream cancel relay; otherwise emits local cancelled batch fallback |
| `/v1/vector_stores` | Implemented | Stateful mode supports OpenAI-compatible upstream create and list relay; otherwise emits local vector store fallback |
| `/v1/vector_stores/{vector_store_id}` | Implemented | Stateful mode supports OpenAI-compatible upstream retrieve, update, and delete relay; otherwise emits local vector store metadata or deleted response fallback |
| `/v1/vector_stores/{vector_store_id}/search` | Implemented | Stateful mode supports OpenAI-compatible upstream semantic search relay; otherwise emits local vector store search fallback |
| `/v1/vector_stores/{vector_store_id}/files` | Implemented | Stateful mode supports OpenAI-compatible upstream create and list relay; otherwise emits local vector store file fallback |
| `/v1/vector_stores/{vector_store_id}/files/{file_id}` | Implemented | Stateful mode supports OpenAI-compatible upstream retrieve and delete relay; otherwise emits local vector store file metadata or deleted response fallback |
| `/v1/vector_stores/{vector_store_id}/file_batches` | Implemented | Stateful mode supports OpenAI-compatible upstream create relay; otherwise emits local vector store file batch fallback |
| `/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}` | Implemented | Stateful mode supports OpenAI-compatible upstream retrieve relay; otherwise emits local vector store file batch metadata fallback |
| `/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/cancel` | Implemented | Stateful mode supports OpenAI-compatible upstream cancel relay; otherwise emits local cancelled vector store file batch fallback |
| `/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/files` | Implemented | Stateful mode supports OpenAI-compatible upstream file listing relay; otherwise emits local vector store batch file list fallback |
| `/v1/videos` | Implemented | Stateful mode supports OpenAI-compatible upstream create and list relay; otherwise emits local video fallback |
| `/v1/videos/{video_id}` | Implemented | Stateful mode supports OpenAI-compatible upstream retrieve and delete relay; otherwise emits local video metadata or deleted response fallback |
| `/v1/videos/{video_id}/content` | Implemented | Stateful mode supports OpenAI-compatible upstream binary passthrough; otherwise emits local video-content fallback |
| `/v1/videos/{video_id}/remix` | Implemented | Stateful mode supports OpenAI-compatible upstream remix relay; otherwise emits local remixed video fallback |

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
| Files | Defined | Implemented (`create`, `list`, `retrieve`, `delete`, `content`) |
| Uploads | Defined | Implemented |
| Audio | Defined | Implemented |
| Fine Tuning | Defined | Implemented |
| Realtime | Defined | Implemented |
| Assistants | Defined | Implemented (`create`, `list`, `retrieve`, `update`, `delete`) |
| Vector Stores | Defined | Implemented |
| Batches | Defined | Implemented |
| Videos | Defined | Implemented (`create`, `list`, `retrieve`, `delete`, `content`, `remix`) |
| Webhooks | Defined | Implemented (`create`, `list`, `retrieve`, `update`, `delete`) |
| Evals | Defined | Implemented |

## Runtime Behavior Notes

| Capability | Current Behavior |
|---|---|
| Upstream proxying | Implemented across all currently defined contract families; stateful gateway relays OpenAI-compatible chat, chat SSE, completions, responses, embeddings, files create/list/retrieve/delete/content, upload create/part/complete/cancel, moderations, image generations, videos create/list/retrieve/delete/content/remix, audio transcriptions, audio translations, audio speech binary passthrough, fine-tuning jobs create/list/retrieve/cancel, assistants create/list/retrieve/update/delete, webhooks create/list/retrieve/update/delete, realtime sessions, evals, batches create/list/retrieve/cancel, and vector stores create/list/retrieve/update/delete/search plus vector store files create/list/retrieve/delete plus vector store file batches create/retrieve/cancel/list-files when provider and credential records are configured |
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
