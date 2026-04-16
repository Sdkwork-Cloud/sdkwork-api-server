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
- `code.gemini`: Gemini mirror routes on the official `/v1beta/models/{model}:*` surface
- `images.openai`: OpenAI image mirror routes on the official `/v1/images/*` surface
- the public contract does not invent wrapper prefixes such as `/code/*`, `/claude/*`, or `/gemini/*`

## Route Families

OpenAI-family rows below use the official `/v1` prefix. Claude and Gemini keep their official provider paths instead of being remapped into a gateway-specific namespace.

| Family | Routes | Notes |
|---|---|---|
| models | `GET /models`, `GET /models/{model_id}`, `DELETE /models/{model_id}` | catalog-backed in stateful mode |
| chat completions | `GET /chat/completions`, `POST /chat/completions`, `GET/POST/DELETE /chat/completions/{completion_id}`, `GET /chat/completions/{completion_id}/messages` | supports compatible JSON and stream relay |
| completions | `POST /completions` | legacy text completion surface |
| responses | `POST /responses`, `POST /responses/input_tokens`, `POST /responses/compact`, `GET/DELETE /responses/{response_id}`, `GET /responses/{response_id}/input_items`, `POST /responses/{response_id}/cancel` | OpenAI-style response workflow surface |
| embeddings | `POST /embeddings` | request-model-driven provider selection |
| moderations | `POST /moderations` | OpenAI-compatible moderation route |
| images | `POST /images/generations`, `POST /images/edits`, `POST /images/variations` | active public mirror is `images.openai`; provider routing can vary behind the shared OpenAI image contract |
| audio | `POST /audio/transcriptions`, `POST /audio/translations`, `POST /audio/speech`, `GET /audio/voices`, `POST /audio/voice_consents` | includes binary speech output and voice consent creation |
| files | `GET/POST /files`, `GET/DELETE /files/{file_id}`, `GET /files/{file_id}/content` | metadata plus binary content retrieval |
| uploads | `POST /uploads`, `POST /uploads/{upload_id}/parts`, `POST /uploads/{upload_id}/complete`, `POST /uploads/{upload_id}/cancel` | multipart upload lifecycle |
| containers | `GET/POST /containers`, `GET/DELETE /containers/{container_id}`, `GET/POST /containers/{container_id}/files`, `GET/DELETE /containers/{container_id}/files/{file_id}`, `GET /containers/{container_id}/files/{file_id}/content` | container and nested file flows |
| assistants | `GET/POST /assistants`, `GET/POST/DELETE /assistants/{assistant_id}` | compatible assistants management |
| threads | `POST /threads`, `GET/POST/DELETE /threads/{thread_id}`, nested messages and runs routes | includes tool output submission and run steps |
| conversations | `GET/POST /conversations`, `GET/POST/DELETE /conversations/{conversation_id}`, nested item routes | conversation-native flow for response-style workloads |
| vector stores | `GET/POST /vector_stores`, `GET/POST/DELETE /vector_stores/{vector_store_id}`, nested search, files, and file batches | retrieval and ingestion workflows |
| batches | `GET/POST /batches`, `GET /batches/{batch_id}`, `POST /batches/{batch_id}/cancel` | asynchronous batch workflows |
| fine tuning | `GET/POST /fine_tuning/jobs`, retrieve, cancel, events, checkpoints, pause, resume, checkpoint permissions | broad fine-tuning family coverage |
| webhooks | `GET/POST /webhooks`, `GET/POST/DELETE /webhooks/{webhook_id}` | compatible webhook CRUD |
| realtime | `POST /realtime/sessions` | realtime session creation |
| evals | `GET/POST /evals`, `GET/POST/DELETE /evals/{eval_id}`, nested runs and output item routes | evaluation workflows |
| videos | `GET/POST /videos`, retrieve, delete, content, remix, edits, extensions, extend, and character routes | includes both canonical and nested video resources |
| music | `GET/POST /music`, `GET/DELETE /music/{music_id}`, `GET /music/{music_id}/content`, `POST /music/lyrics` | resource-oriented music generation, retrieval, binary content fetch, and lyrics creation |
| market | `GET /market/products`, `GET /market/offers`, `POST /market/quotes` | public API product catalog, offer discovery, and quote workflows |
| marketing | `POST /marketing/coupons/validate`, `POST /marketing/coupons/reserve`, `POST /marketing/coupons/confirm`, `POST /marketing/coupons/rollback` | coupon-first validation, reservation, redemption, and rollback surface |
| commercial | `GET /commercial/account`, `GET /commercial/account/benefit-lots` | commercial account summary plus benefit-lot traversal and coupon/account-arrival evidence |

Phase 2A keeps the active image mirror contract on the shared OpenAI image routes `/v1/images*` and publishes that family as `images.openai`. Reserved future image mirror families such as `images.nanobanana`, `images.midjourney`, `images.volcengine`, `images.aliyun`, and `images.kling` remain design-time names only until their official protocols are formalized and implemented.

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
