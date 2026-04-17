# Gateway API

The gateway service exposes mirror-style public APIs. Official client paths stay unchanged, so existing SDKs and CLIs should only need to switch the `base_url`.

## Base URL and Auth

- default local base URL: `http://127.0.0.1:8080`
- primary auth: `Authorization: Bearer skw_live_...`
- health: `GET /health`
- metrics: `GET /metrics`
- OpenAPI JSON: `GET /openapi.json`
- API inventory UI: `GET /docs`

Minimal first request:

```bash
curl http://127.0.0.1:8080/v1/models \
  -H "Authorization: Bearer skw_live_your_key_here"
```

In standalone service mode, expect stateful gateway behavior backed by the admin store. The stateless gateway runtime is available as a library/runtime shape and remains documented through the compatibility matrix.

OpenAPI is generated from the current `axum` route implementation, so the JSON document and the browser page stay aligned with the live router surface.

## Mirror Protocol Families

- `code.openai`: OpenAI and Codex mirror routes on the official `/v1/*` surface
- `code.claude`: Claude mirror routes on the official `/v1/messages*` surface
- `code.gemini`: Gemini mirror routes on the official `/v1beta/models/{model}:*` surface, including image-capable Gemini models such as Nano Banana
- `images.openai`: OpenAI image mirror routes on the official `/v1/images/*` surface
- `images.kling`: Kling image mirror routes on the shared official DashScope `/api/v1/services/aigc/image-generation/*` and `/api/v1/tasks/{task_id}` surface
- `images.aliyun`: Aliyun image mirror routes on the shared official DashScope `/api/v1/services/aigc/image-generation/*` and `/api/v1/tasks/{task_id}` surface
- `images.volcengine`: Volcengine image mirror routes on the official `/api/v3/images/generations` surface
- `audio.openai`: Shared audio mirror routes on the official `/v1/audio/*` surface
- `video.openai`: Shared video mirror routes on the official `/v1/videos*` surface, including Sora 2 and Sora 2 Pro
- `video.kling`: Kling video mirror routes on the shared official DashScope `/api/v1/services/aigc/video-generation/*` and `/api/v1/tasks/{task_id}` surface
- `video.aliyun`: Aliyun video mirror routes on the shared official DashScope `/api/v1/services/aigc/video-generation/*` and `/api/v1/tasks/{task_id}` surface
- `video.google-veo`: Google Veo mirror routes on the official Vertex AI publisher-model `/v1/projects/*/locations/*/publishers/google/models/*` surface, including Veo 3-class models selected by the official model path
- `video.minimax`: MiniMax video mirror routes on the official `/v1/video_generation`, `/v1/query/video_generation`, and `/v1/files/retrieve` surface
- `video.vidu`: Vidu video mirror routes on the official `/ent/v2/*` surface
- `video.volcengine`: Volcengine video mirror routes on the official `/api/v1/contents/generations/tasks*` surface
- `music.openai`: Shared music mirror routes on the official `/v1/music*` surface
- `music.google`: Google music mirror routes on the official Vertex AI `/v1/projects/*/locations/*/publishers/google/models/{model}:predict` surface
- `music.minimax`: MiniMax music mirror routes on the official `/v1/music_generation` and `/v1/lyrics_generation` surface
- `music.suno`: Suno music mirror routes on the official `/api/v1/*` surface
- the public contract does not invent wrapper prefixes such as `/code/*`, `/claude/*`, or `/gemini/*`

## OpenAPI Groups

The table below uses the exact OpenAPI tag names exposed by `/openapi.json`. Shared default families come first, followed by provider-specific mirror families that keep the official upstream paths unchanged.

| OpenAPI Tag | Routes | Notes |
|---|---|---|
| `system.sdkwork` | `GET /health` | system-facing health route surfaced in OpenAPI; `GET /metrics` remains an operational endpoint outside the public OpenAPI inventory |
| `code.openai` | `GET /models`, `GET /models/{model_id}`, `DELETE /models/{model_id}`, `GET/POST /chat/completions`, `GET/POST/DELETE /chat/completions/{completion_id}`, `GET /chat/completions/{completion_id}/messages`, `POST /completions`, `POST /responses`, `POST /responses/input_tokens`, `POST /responses/compact`, `GET/DELETE /responses/{response_id}`, `GET /responses/{response_id}/input_items`, `POST /responses/{response_id}/cancel`, `POST /embeddings`, `POST /moderations` | shared OpenAI and Codex mirror family on the official `/v1/*` contract |
| `code.claude` | `POST /v1/messages`, `POST /v1/messages/count_tokens` | official Claude mirror family for Claude Code and other Anthropic clients |
| `code.gemini` | `POST /v1beta/models/{model}:generateContent`, `POST /v1beta/models/{model}:streamGenerateContent?alt=sse`, `POST /v1beta/models/{model}:countTokens` | official Gemini mirror family, including image-capable Gemini models such as Nano Banana |
| `images.openai` | `POST /images/generations`, `POST /images/edits`, `POST /images/variations` | shared default image mirror family on the official OpenAI `/v1/images/*` contract |
| `images.kling` | `POST /api/v1/services/aigc/image-generation/generation`, `GET /api/v1/tasks/{task_id}` | provider-specific image mirror family for Kling-compatible clients on the shared official DashScope async image transport |
| `images.aliyun` | `POST /api/v1/services/aigc/image-generation/generation`, `GET /api/v1/tasks/{task_id}` | provider-specific image mirror family for Aliyun-compatible clients on the shared official DashScope async image transport |
| `images.volcengine` | `POST /api/v3/images/generations` | provider-specific image mirror family on Volcengine Ark's official image generation transport |
| `audio.openai` | `POST /audio/transcriptions`, `POST /audio/translations`, `POST /audio/speech`, `GET /audio/voices`, `POST /audio/voice_consents` | shared default audio mirror family on the official `/v1/audio/*` contract |
| `files` | `GET/POST /files`, `GET/DELETE /files/{file_id}`, `GET /files/{file_id}/content` | metadata plus binary content retrieval |
| `uploads` | `POST /uploads`, `POST /uploads/{upload_id}/parts`, `POST /uploads/{upload_id}/complete`, `POST /uploads/{upload_id}/cancel` | multipart upload lifecycle |
| `containers` | `GET/POST /containers`, `GET/DELETE /containers/{container_id}`, `GET/POST /containers/{container_id}/files`, `GET/DELETE /containers/{container_id}/files/{file_id}`, `GET /containers/{container_id}/files/{file_id}/content` | container and nested file flows |
| `assistants` | `GET/POST /assistants`, `GET/POST/DELETE /assistants/{assistant_id}` | compatible assistants management |
| `threads` | `POST /threads`, `GET/POST/DELETE /threads/{thread_id}`, `GET/POST /threads/{thread_id}/messages`, `GET/POST/DELETE /threads/{thread_id}/messages/{message_id}` | assistant thread and message management |
| `runs` | `POST /threads/runs`, `GET/POST /threads/{thread_id}/runs`, `GET/POST /threads/{thread_id}/runs/{run_id}`, `POST /threads/{thread_id}/runs/{run_id}/cancel`, `POST /threads/{thread_id}/runs/{run_id}/submit_tool_outputs`, `GET /threads/{thread_id}/runs/{run_id}/steps`, `GET /threads/{thread_id}/runs/{run_id}/steps/{step_id}` | assistant run orchestration and run-step surface |
| `conversations` | `GET/POST /conversations`, `GET/POST/DELETE /conversations/{conversation_id}`, `GET/POST /conversations/{conversation_id}/items`, `GET/DELETE /conversations/{conversation_id}/items/{item_id}` | conversation-native flow for response-style workloads |
| `vector-stores` | `GET/POST /vector_stores`, `GET/POST/DELETE /vector_stores/{vector_store_id}`, nested search, files, and file batches | retrieval and ingestion workflows |
| `batches` | `GET/POST /batches`, `GET /batches/{batch_id}`, `POST /batches/{batch_id}/cancel` | asynchronous batch workflows |
| `fine-tuning` | `GET/POST /fine_tuning/jobs`, retrieve, cancel, events, checkpoints, pause, resume, and checkpoint permissions | broad fine-tuning family coverage |
| `webhooks` | `GET/POST /webhooks`, `GET/POST/DELETE /webhooks/{webhook_id}` | compatible webhook CRUD |
| `realtime` | `POST /realtime/sessions` | realtime session creation |
| `evals` | `GET/POST /evals`, `GET/POST/DELETE /evals/{eval_id}`, nested runs and output item routes | evaluation workflows |
| `video.openai` | `GET/POST /videos`, retrieve, delete, content, remix, edits, extensions, extend, and character routes | shared default video mirror family on the official `/v1/videos*` contract, including Sora 2 and Sora 2 Pro |
| `video.kling` | `POST /api/v1/services/aigc/video-generation/video-synthesis`, `GET /api/v1/tasks/{task_id}` | provider-specific video mirror family for Kling-compatible clients on the shared official DashScope async transport |
| `video.aliyun` | `POST /api/v1/services/aigc/video-generation/video-synthesis`, `GET /api/v1/tasks/{task_id}` | provider-specific video mirror family for Aliyun-compatible clients on the shared official DashScope async transport |
| `video.google-veo` | `POST /v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predictLongRunning`, `POST /v1/projects/{project}/locations/{location}/publishers/google/models/{model}:fetchPredictOperation` | provider-specific Google Veo mirror family, including Veo 3-class models selected through `{model}` |
| `video.minimax` | `POST /v1/video_generation`, `GET /v1/query/video_generation`, `GET /v1/files/retrieve` | provider-specific MiniMax video mirror family on the official async video transport |
| `video.vidu` | `POST /ent/v2/text2video`, `POST /ent/v2/img2video`, `POST /ent/v2/reference2video`, `GET /ent/v2/tasks/{id}/creations`, `POST /ent/v2/tasks/{id}/cancel` | provider-specific Vidu video mirror family on the official async video transport |
| `video.volcengine` | `POST /api/v1/contents/generations/tasks`, `GET /api/v1/contents/generations/tasks/{id}` | provider-specific Volcengine video mirror family on the official async task transport |
| `music.openai` | `GET/POST /music`, `GET/DELETE /music/{music_id}`, `GET /music/{music_id}/content`, `POST /music/lyrics` | shared default music mirror family on the official `/v1/music*` contract |
| `music.google` | `POST /v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict` | provider-specific Google music mirror family on the official Vertex AI predict path |
| `music.minimax` | `POST /v1/music_generation`, `POST /v1/lyrics_generation` | provider-specific MiniMax music mirror family on the official generation paths |
| `music.suno` | `POST /api/v1/generate`, `GET /api/v1/generate/record-info`, `POST /api/v1/lyrics`, `GET /api/v1/lyrics/record-info` | provider-specific Suno music mirror family on the official transport paths |
| `market` | `GET /market/products`, `GET /market/offers`, `POST /market/quotes` | public API product catalog, offer discovery, and quote workflows |
| `marketing` | `POST /marketing/coupons/validate`, `POST /marketing/coupons/reserve`, `POST /marketing/coupons/confirm`, `POST /marketing/coupons/rollback` | coupon-first validation, reservation, redemption, and rollback surface |
| `commercial` | `GET /commercial/account`, `GET /commercial/account/benefit-lots` | commercial account summary plus benefit-lot traversal and coupon/account-arrival evidence |

The gateway now publishes four active image mirror tags across three public path families: the shared `images.openai` contract on `/v1/images*`, the provider-specific `images.kling` and `images.aliyun` tags on the shared official DashScope `/api/v1/services/aigc/image-generation/generation` and `/api/v1/tasks/{task_id}` paths, and the provider-specific `images.volcengine` tag on Volcengine Ark's official `/api/v3/images/generations` path. Nano Banana stays on Google's official Gemini `/v1beta/models/{model}:generateContent` surface and is therefore documented under `code.gemini` instead of a separate `images.nanobanana` family. `images.midjourney` remains unpublished because Midjourney does not provide an official API surface that can be mirrored by switching only `base_url`.

This slice keeps the active audio mirror contract on the shared `/v1/audio/*` routes and publishes that family as `audio.openai`. The public contract stays on the current shared audio surface and does not introduce wrapper paths such as `/audio/openai/*`.

The gateway now publishes seven active video mirror families: the shared `video.openai` contract on `/v1/videos*` for OpenAI video clients, including Sora 2 and Sora 2 Pro, the provider-specific `video.kling` and `video.aliyun` contracts on the shared official DashScope `/api/v1/services/aigc/video-generation/video-synthesis` and `/api/v1/tasks/{task_id}` paths, the provider-specific `video.google-veo` contract on Google Vertex AI's official `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predictLongRunning` and `:fetchPredictOperation` paths, including Veo 3-class models selected through `{model}`, the provider-specific `video.minimax` contract on MiniMax's official `/v1/video_generation`, `/v1/query/video_generation`, and `/v1/files/retrieve` paths, the provider-specific `video.vidu` contract on Vidu's official `/ent/v2/text2video`, `/ent/v2/img2video`, `/ent/v2/reference2video`, `/ent/v2/tasks/{id}/creations`, and `/ent/v2/tasks/{id}/cancel` paths, and the provider-specific `video.volcengine` contract on Volcengine's official `/api/v1/contents/generations/tasks` and `/api/v1/contents/generations/tasks/{id}` paths. Because OpenAI already defines the official Sora transport, the gateway does not publish a separate `video.sora` family.

The gateway now publishes four active music mirror families: the shared `music.openai` contract on `/v1/music*`, the provider-specific `music.google` contract on Google Vertex AI's official `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict` path, the provider-specific `music.minimax` contract on MiniMax's official `/v1/music_generation` and `/v1/lyrics_generation` paths, and the provider-specific `music.suno` contract on Suno's official `/api/v1/generate*` and `/api/v1/lyrics*` paths.

## Gateway Semantics

- public contract rule: preserve the official client path and switch only the gateway base URL
- provider selection is routed through models, route keys, and routing policy
- usage and billing are recorded against authenticated projects in stateful mode
- create-like routes may preserve route-key-based provider selection while recording usage against created resource IDs
- generation-style routes such as chat, completions, responses, embeddings, and moderations keep billing keyed to the request model even when upstream responses return resource IDs
- generation-style media routes such as images, videos, and music keep billing keyed to the request model while preserving created resource IDs as downstream references
- commercial benefit-lot traversal supports `after_lot_id` and `limit`, and returns `page.after_lot_id`, `page.next_after_lot_id`, `page.has_more`, and `page.returned_count`
- coupon-to-account-arrival evidence stays explicit through `scope_order_id` on `GET /commercial/account/benefit-lots`

## Helpful Headers

| Header | Purpose |
|---|---|
| `Authorization` | gateway API key |
| `Content-Type` | JSON, multipart, or upstream-compatible media type |
| `x-request-id` | request correlation |
| `x-sdkwork-region` | optional routing hint for geo-affinity-aware provider selection |

## Related Docs

- public contract and execution truth:
  - [API Compatibility](/reference/api-compatibility)
  - [Full Compatibility Matrix](/api/compatibility-matrix)
- control plane:
  - [Admin API](/api-reference/admin-api)
