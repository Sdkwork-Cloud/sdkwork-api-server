# Compatibility Matrix

## Compatibility Levels

The gateway now uses five execution-truth labels instead of a binary `implemented / not implemented` flag.

| Level | Meaning |
|---|---|
| `native` | Implemented directly by SDKWork and backed by local control-plane or runtime state |
| `relay` | Forwarded to an upstream OpenAI-compatible provider when the gateway is configured with provider, credential, and model data |
| `translated` | Accepted by the gateway but mapped to a different upstream capability or execution primitive |
| `emulated` | Returned locally in a compatible shape without real upstream execution |
| `unsupported` | Contract is not available in the current runtime |

## Data Plane

The table below reflects the current runtime truth as of 2026-03-14.

| API Family | Stateful Gateway | Stateless Gateway | Notes |
|---|---|---|---|
| `/v1/models` | `native` | `emulated` | Stateful mode reads the local catalog; stateless mode returns a compatible local list |
| `/v1/chat/completions` | `relay` | `emulated` | Supports JSON and SSE relay for configured OpenAI-compatible upstreams |
| `/v1/completions` | `relay` | `emulated` | Relays legacy text completions when provider wiring exists |
| `/v1/responses` | `relay` | `emulated` | Stateful mode relays create, retrieve, delete, cancel, compact, input item flows, and SSE streaming |
| `/v1/embeddings` | `relay` | `emulated` | Uses catalog, credential, and provider state in stateful mode |
| `/v1/files` | `relay` | `emulated` | Stateful mode relays multipart upload, metadata, and binary content |
| `/v1/uploads` | `relay` | `emulated` | Upload creation, part upload, completion, and cancel relay in stateful mode |
| `/v1/audio/*` | `relay` | `emulated` | Speech can relay binary or event-stream output |
| `/v1/images/*` | `relay` | `emulated` | Generations, edits, and variations relay in stateful mode |
| `/v1/moderations` | `relay` | `emulated` | Stateful mode relays provider moderation calls |
| `/v1/realtime/sessions` | `relay` | `emulated` | Compatible request/response contract is present in both modes |
| `/v1/assistants` | `relay` | `emulated` | Stateful mode relays create, list, retrieve, update, and delete |
| `/v1/threads` | `relay` | `emulated` | Includes messages, runs, run steps, cancel, and tool output submission |
| `/v1/conversations` | `relay` | `emulated` | Includes conversation items CRUD-compatible flows |
| `/v1/vector_stores` | `relay` | `emulated` | Includes search, files, and file batch flows |
| `/v1/batches` | `relay` | `emulated` | Create, list, retrieve, and cancel are wired in stateful mode |
| `/v1/fine_tuning/jobs` | `relay` | `emulated` | Create, list, retrieve, and cancel are relay-capable |
| `/v1/webhooks` | `relay` | `emulated` | CRUD-compatible relay path when upstream supports the same contract |
| `/v1/evals` | `relay` | `emulated` | Stateful mode relays eval creation/listing-compatible flow |
| `/v1/videos` | `relay` | `emulated` | Create, list, retrieve, content, delete, and remix relay in stateful mode |

## Control Plane

Admin APIs are SDKWork-owned control-plane surfaces and therefore classify as `native`.

| Endpoint Family | Level | Notes |
|---|---|---|
| `/admin/auth/*` | `native` | Signed JWT login plus authenticated caller inspection |
| `/admin/tenants` | `native` | SQLite and PostgreSQL backed |
| `/admin/projects` | `native` | SQLite and PostgreSQL backed |
| `/admin/api-keys` | `native` | Gateway API key issuance plus tenancy-aware lookup |
| `/admin/channels` | `native` | Control-plane definition of upstream ecosystems |
| `/admin/providers` | `native` | Supports multi-channel bindings plus adapter and base URL config |
| `/admin/models` | `native` | Stores model capability metadata and streaming flags |
| `/admin/credentials` | `native` | Secret references with encrypted persistence backends |
| `/admin/extensions/packages` | `native` | Lists discovered extension package manifests from configured search paths with normalized package naming and validation details |
| `/admin/extensions/installations` | `native` | Stores extension installation state and config payload |
| `/admin/extensions/instances` | `native` | Stores mounted extension instances with runtime config |
| `/admin/extensions/runtime-statuses` | `native` | Lists normalized runtime status for active connector and native dynamic runtimes currently tracked by the host |
| `/admin/routing/simulations` | `native` | Catalog-backed routing decision preview |
| `/admin/usage/records` | `native` | Lists gateway-recorded usage |
| `/admin/billing/ledger` | `native` | Lists gateway-booked billing entries |

## Extension Runtime

| Runtime Mode | Level | Notes |
|---|---|---|
| `builtin` | `native` | Active today through `sdkwork-api-extension-host` and built-in provider factories |
| `native_dynamic` | `native` | Trusted provider packages can now load through the JSON ABI, manifest verification, dynamic library symbol resolution, optional `init` or `health_check` or `shutdown` lifecycle hooks, and callback-based stream execution for `/v1/chat/completions`, `/v1/responses`, `/v1/audio/speech`, `/v1/files/{file_id}/content`, and `/v1/videos/{video_id}/content` |
| `connector` | `native` | Managed process lifecycle is active in the host, with HTTP health checks, reusable external endpoint attachment, protocol-mapped relay through the current adapter set, and trust-policy gating for discovered external packages |

## Current Built-In Extension IDs

| Extension ID | Kind | Runtime | Notes |
|---|---|---|---|
| `sdkwork.provider.openai.official` | `provider` | `builtin` | OpenAI-compatible direct upstream |
| `sdkwork.provider.openrouter` | `provider` | `builtin` | OpenRouter-compatible upstream |
| `sdkwork.provider.ollama` | `provider` | `builtin` | Local Ollama-compatible upstream |
