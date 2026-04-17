# Compatibility Matrix

## Public Contract Rules

- public clients should only switch the gateway `base_url`; official provider paths stay unchanged
- execution-truth labels describe runtime behavior and do not replace the public mirror protocol taxonomy
- the gateway does not publish wrapper prefixes such as `/code/*`, `/claude/*`, or `/gemini/*`

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

The table below reflects the current runtime truth as of 2026-04-16.

| API Family | Stateful Gateway | Stateless Gateway | Notes |
|---|---|---|---|
| `/v1/models` | `native` | `relay` | Stateful mode reads the local catalog; stateless mode relays model list and retrieve when a stateless upstream runtime is configured and otherwise falls back to a compatible local catalog |
| `/v1/chat/completions` | `relay` | `relay` | Supports JSON and SSE relay for configured OpenAI-compatible upstreams; stateless mode now also relays list, retrieve, update, delete, and messages list when one upstream runtime is configured |
| `/v1/messages` | `translated` | `translated` | Claude mirror protocol for Claude Code and other Anthropic clients. Requests translate into the shared chat-completions flow; stateful mode accepts `Authorization` and `x-api-key`, while both modes now preserve `anthropic-version` and `anthropic-beta` on the upstream relay path |
| `/v1/messages/count_tokens` | `translated` | `translated` | Claude token counting mirror protocol. Uses the shared response-token counting path and preserves the configured model route key for usage accounting in stateful mode |
| `/v1/completions` | `relay` | `relay` | Relays legacy text completions when provider wiring exists; stateless mode uses the configured single-upstream runtime or falls back locally |
| `/v1/responses` | `relay` | `relay` | Stateful mode relays create, retrieve, delete, cancel, compact, input item flows, SSE streaming, and project quota admission; stateless mode relays the same core response operations through its configured upstream runtime |
| `/v1beta/models/{model}:generateContent` | `translated` | `translated` | Gemini mirror protocol for Gemini CLI gateway mode, Google Generative Language clients, and image-capable Gemini clients such as Nano Banana. Requests translate into the shared chat-completions flow; stateful mode accepts `Authorization`, `x-goog-api-key`, and `?key=` while preserving routing, quota, billing, usage recording, and the official `GOOGLE_GEMINI_BASE_URL` plus `GEMINI_API_KEY_AUTH_MECHANISM=bearer` client setup path |
| `/v1beta/models/{model}:streamGenerateContent` | `translated` | `translated` | Gemini SSE mirror protocol via `?alt=sse`, with OpenAI-compatible upstream chunk streams re-emitted as Gemini event frames |
| `/v1beta/models/{model}:countTokens` | `translated` | `translated` | Gemini token counting mirror protocol through the shared token-count execution path |
| `/v1/embeddings` | `relay` | `relay` | Uses catalog, credential, and provider state in stateful mode; stateless mode relays embeddings to its configured upstream runtime or falls back locally |
| `/v1/containers` | `relay` | `relay` | Container create, list, retrieve, delete, container file create, list, retrieve, delete, and binary content relay are now wired in both modes |
| `/v1/files` | `relay` | `relay` | Stateful mode relays multipart upload, metadata, and binary content; stateless mode relays the same surface through its configured upstream runtime or falls back locally |
| `/v1/uploads` | `relay` | `relay` | Upload creation, part upload, completion, and cancel relay in stateful mode; stateless mode relays the same upload surface through its configured upstream runtime or falls back locally |
| `/v1/audio/*` | `relay` | `relay` | Speech can relay binary or event-stream output; both modes now also cover transcription, translation, voices listing, and voice consent flows through the configured upstream runtime or local compatible fallback |
| `/v1/images/*` | `relay` | `relay` | Generations, edits, and variations relay in stateful mode; stateless mode relays the same image operations through its configured upstream runtime or falls back locally |
| `/api/v1/services/aigc/image-generation/generation` | `relay` | `relay` | Provider-specific shared DashScope image mirror transport for `images.kling` and `images.aliyun`; stateful mode preserves provider identity and task ownership while stateless mode relays the same official path directly |
| `/api/v3/images/generations` | `relay` | `relay` | Provider-specific Volcengine image mirror transport for `images.volcengine` on the official Ark image-generation path |
| `/api/v1/tasks/{task_id}` | `relay` | `relay` | Shared DashScope async task lookup used by the active `images.kling`, `images.aliyun`, `video.kling`, and `video.aliyun` mirror families |
| `/v1/moderations` | `relay` | `relay` | Stateful mode relays provider moderation calls; stateless mode relays moderation through its configured upstream runtime or falls back locally |
| `/v1/realtime/sessions` | `relay` | `relay` | Compatible request/response contract is present in both modes; stateless mode now relays realtime session creation through its configured upstream runtime or falls back locally |
| `/v1/assistants` | `relay` | `relay` | Stateful mode relays create, list, retrieve, update, and delete; stateless mode relays the same assistants surface through its configured upstream runtime or falls back locally |
| `/v1/threads` | `relay` | `relay` | Includes messages, runs, run steps, cancel, and tool output submission; stateless mode relays the same nested thread flows through its configured upstream runtime or falls back locally |
| `/v1/conversations` | `relay` | `relay` | Includes conversation items CRUD-compatible flows; stateless mode relays the same conversation and item flows through its configured upstream runtime or falls back locally |
| `/v1/vector_stores` | `relay` | `relay` | Includes search, files, and file batch flows; stateless mode relays the same vector store surface through its configured upstream runtime or falls back locally |
| `/v1/batches` | `relay` | `relay` | Create, list, retrieve, and cancel are wired in stateful mode; stateless mode relays the same batch operations through its configured upstream runtime or falls back locally |
| `/v1/fine_tuning/jobs` | `relay` | `relay` | Create, list, retrieve, cancel, events, checkpoints, pause, resume, checkpoint permission create or list or delete are relay-capable; stateless mode relays the same fine-tuning surface through its configured upstream runtime or falls back locally |
| `/v1/webhooks` | `relay` | `relay` | CRUD-compatible relay path when upstream supports the same contract; stateless mode relays the same webhook surface through its configured upstream runtime or falls back locally |
| `/v1/evals` | `relay` | `relay` | Create, list, retrieve, update, delete, run list, run create, run retrieve, run delete, run cancel, output item list, and output item retrieve are relay-capable in both modes |
| `/v1/videos` | `relay` | `relay` | Create, list, retrieve, content, delete, remix, official characters create or retrieve, official edits, official extensions, legacy nested character aliases, character update, and extend relay are available in both modes |
| `/api/v1/services/aigc/video-generation/video-synthesis` | `relay` | `relay` | Provider-specific shared DashScope video mirror transport for `video.kling` and `video.aliyun`; both modes preserve the official async synthesis path |
| `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predictLongRunning` | `relay` | `relay` | Provider-specific Google Veo mirror transport for `video.google-veo`, including Veo 3-class models selected through `{model}` |
| `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:fetchPredictOperation` | `relay` | `relay` | Provider-specific Google Veo operation lookup transport for `video.google-veo`, including Veo 3-class models selected through `{model}` |
| `/api/v1/contents/generations/tasks*` | `relay` | `relay` | Provider-specific Volcengine video task transport for `video.volcengine` on the official create and retrieve task paths |
| `/v1/video_generation` | `relay` | `relay` | Provider-specific MiniMax video generation transport for `video.minimax` |
| `/v1/query/video_generation` | `relay` | `relay` | Provider-specific MiniMax video query transport for `video.minimax` |
| `/v1/files/retrieve` | `relay` | `relay` | Provider-specific MiniMax file retrieval transport used by `video.minimax` result downloads |
| `/ent/v2/text2video` | `relay` | `relay` | Provider-specific Vidu text-to-video transport for `video.vidu` |
| `/ent/v2/img2video` | `relay` | `relay` | Provider-specific Vidu image-to-video transport for `video.vidu` |
| `/ent/v2/reference2video` | `relay` | `relay` | Provider-specific Vidu reference-video transport for `video.vidu` |
| `/ent/v2/tasks/{id}/creations` | `relay` | `relay` | Provider-specific Vidu task result lookup transport for `video.vidu` |
| `/ent/v2/tasks/{id}/cancel` | `relay` | `relay` | Provider-specific Vidu task cancel transport for `video.vidu` |
| `/v1/music*` | `relay` | `relay` | Shared music mirror transport for `music.openai`, including list, create, retrieve, delete, content, and lyrics operations |
| `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict` | `relay` | `relay` | Provider-specific Google music mirror transport for `music.google` on the official Vertex AI predict path |
| `/v1/music_generation` | `relay` | `relay` | Provider-specific MiniMax music generation transport for `music.minimax` |
| `/v1/lyrics_generation` | `relay` | `relay` | Provider-specific MiniMax lyrics generation transport for `music.minimax` |
| `/api/v1/generate*` | `relay` | `relay` | Provider-specific Suno generation transport for `music.suno`, including create and record-info lookup |
| `/api/v1/lyrics*` | `relay` | `relay` | Provider-specific Suno lyrics transport for `music.suno`, including create and record-info lookup |

## Control Plane

Admin APIs are SDKWork-owned control-plane surfaces and therefore classify as `native`.

| Endpoint Family | Level | Notes |
|---|---|---|
| `/admin/auth/*` | `native` | Signed JWT login plus authenticated caller inspection |
| `/portal/auth/*` | `native` | Public self-service registration, login, and authenticated caller inspection with a dedicated portal JWT boundary |
| `/portal/workspace` | `native` | Returns the caller-owned default tenant and project workspace summary |
| `/portal/api-keys` | `native` | Self-service gateway API key issuance and scoped listing for the caller-owned workspace |
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
| `/admin/extensions/runtime-reloads` | `native` | Triggers an explicit managed-runtime reload for all runtimes, one extension family, or one connector instance, rebuilds trusted extension discovery state with unrelated native-dynamic runtime reuse, and returns the fresh runtime status snapshot plus applied-scope metadata and discovery counts |
| `/admin/routing/health-snapshots` | `native` | Lists persisted provider health snapshots captured from live runtime status or active builtin upstream probes for admin observability and routing fallback |
| `/admin/routing/decision-logs` | `native` | Lists persisted gateway and admin-simulation routing decisions, including SLO degraded state, requested-region geo-affinity evidence, and per-candidate assessment details |
| `/admin/routing/simulations` | `native` | Catalog-backed routing decision preview with candidate assessment, runtime health, requested-region geo-affinity, selection reasons, SLO compliance metadata, and persisted audit logging |
| `/admin/usage/records` | `native` | Lists gateway-recorded usage |
| `/admin/usage/summary` | `native` | Aggregated usage counts by project, provider, and model for operator dashboards |
| `/admin/billing/ledger` | `native` | Lists gateway-booked billing entries |
| `/admin/billing/summary` | `native` | Aggregated billing, quota, and exhaustion posture by project |
| `/admin/billing/quota-policies` | `native` | Creates and lists project-scoped hard quota policies used by stateful admission control |

## Extension Runtime

| Runtime Mode | Level | Notes |
|---|---|---|
| `builtin` | `native` | Active today through `sdkwork-api-extension-host` and built-in provider factories |
| `native_dynamic` | `native` | Trusted provider packages can now load through the JSON ABI, manifest verification, dynamic library symbol resolution, optional `init` or `health_check` or `shutdown` lifecycle hooks, and callback-based stream execution for `/v1/chat/completions`, `/v1/responses`, `/v1/audio/speech`, `/v1/files/{file_id}/content`, `/v1/containers/{container_id}/files/{file_id}/content`, and `/v1/videos/{video_id}/content` |
| `connector` | `native` | Managed process lifecycle is active in the host, with HTTP health checks, reusable external endpoint attachment, protocol-mapped relay through the current adapter set, and trust-policy gating for discovered external packages |

## Operational Endpoints

| Endpoint | Level | Notes |
|---|---|---|
| `/health` | `native` | Basic liveness endpoint exposed by gateway, admin, and portal services |
| `/metrics` | `native` | Prometheus-compatible HTTP request counters and duration summaries exposed by gateway, admin, and portal services |

## Operational Conventions

| Convention | Level | Notes |
|---|---|---|
| `x-request-id` | `native` | Gateway and admin routers preserve a caller-supplied request ID or generate one automatically and return it on the response |
| HTTP request tracing | `native` | Standalone binaries initialize structured request logs with service, request ID, method, route, status, and duration fields |

## Current Built-In Extension IDs

| Extension ID | Kind | Runtime | Notes |
|---|---|---|---|
| `sdkwork.provider.openai.official` | `provider` | `builtin` | OpenAI-compatible direct upstream |
| `sdkwork.provider.openrouter` | `provider` | `builtin` | OpenRouter-compatible upstream |
| `sdkwork.provider.ollama` | `provider` | `builtin` | Local Ollama-compatible upstream |
