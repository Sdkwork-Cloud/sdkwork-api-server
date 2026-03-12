# Compatibility Matrix

## Current Implemented Endpoints

| Endpoint | Status | Notes |
|---|---|---|
| `/v1/models` | Implemented | Platform-backed list response |
| `/v1/chat/completions` | Implemented | Basic JSON response and SSE path |
| `/v1/responses` | Implemented | Minimal response object |
| `/v1/embeddings` | Implemented | Minimal list response |

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

## Storage Support

| Driver | Status |
|---|---|
| SQLite | Minimal migration runner implemented |
| PostgreSQL | Boundary crate implemented |
| MySQL | Boundary crate implemented |
| libsql | Boundary crate implemented |
