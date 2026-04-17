# API Compatibility

SDKWork tracks compatibility with five execution-truth labels:

- `native`
- `relay`
- `translated`
- `emulated`
- `unsupported`

These labels describe runtime behavior, not the public URL taxonomy.

## Public Contract Rules

- preserve the official provider path and switch only the gateway `base_url`
- if an OpenAI standard route already exists for a capability, reuse that route as the shared public contract
- if no shared standard exists, expose the provider's official protocol path as a mirror surface
- do not invent wrapper prefixes such as `/code/*`, `/claude/*`, or `/gemini/*`

## Mirror Protocol Families

- `code.openai`: OpenAI and Codex on `/v1/*`
- `code.claude`: Claude on `/v1/messages` and `/v1/messages/count_tokens`
- `code.gemini`: Gemini on `/v1beta/models/{model}:*`, including image-capable Gemini models such as Nano Banana
- `images.openai`: OpenAI image protocol on `/v1/images/*`
- `images.kling`: Kling image protocol on the shared DashScope `/api/v1/services/aigc/image-generation/generation` and `/api/v1/tasks/{task_id}`
- `images.aliyun`: Aliyun image protocol on the shared DashScope `/api/v1/services/aigc/image-generation/generation` and `/api/v1/tasks/{task_id}`
- `images.volcengine`: Volcengine image protocol on `/api/v3/images/generations`
- `audio.openai`: Shared audio protocol on `/v1/audio/*`
- `video.openai`: Shared video protocol on `/v1/videos*`, including Sora 2 and Sora 2 Pro
- `video.kling`: Kling video protocol on the shared DashScope `/api/v1/services/aigc/video-generation/video-synthesis` and `/api/v1/tasks/{task_id}`
- `video.aliyun`: Aliyun video protocol on the shared DashScope `/api/v1/services/aigc/video-generation/video-synthesis` and `/api/v1/tasks/{task_id}`
- `video.google-veo`: Google Veo protocol on Vertex AI's `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predictLongRunning` and `:fetchPredictOperation`, including Veo 3-class models selected through `{model}`
- `video.minimax`: MiniMax video protocol on `/v1/video_generation`, `/v1/query/video_generation`, and `/v1/files/retrieve`
- `video.vidu`: Vidu video protocol on `/ent/v2/text2video`, `/ent/v2/img2video`, `/ent/v2/reference2video`, `/ent/v2/tasks/{id}/creations`, and `/ent/v2/tasks/{id}/cancel`
- `video.volcengine`: Volcengine video protocol on `/api/v1/contents/generations/tasks` and `/api/v1/contents/generations/tasks/{id}`
- `music.openai`: Shared music protocol on `/v1/music*`
- `music.google`: Google music protocol on Vertex AI's `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict`
- `music.minimax`: MiniMax music protocol on `/v1/music_generation` and `/v1/lyrics_generation`
- `music.suno`: Suno music protocol on `/api/v1/generate*` and `/api/v1/lyrics*`

## High-Value API Families

Currently implemented gateway families include:

- `/v1/models`
- `/v1/chat/completions`
- `/v1/messages`
- `/v1/completions`
- `/v1/responses`
- `/v1beta/models/{model}:generateContent`
- `/v1beta/models/{model}:streamGenerateContent`
- `/v1beta/models/{model}:countTokens`
- `/v1/embeddings`
- `/v1/files`
- `/v1/uploads`
- `/v1/audio/*`
- `/v1/images/*`
- `/api/v1/services/aigc/image-generation/generation`
- `/api/v3/images/generations`
- `/api/v1/services/aigc/video-generation/video-synthesis`
- `/api/v1/contents/generations/tasks`
- `/api/v1/contents/generations/tasks/{id}`
- `/api/v1/tasks/{task_id}`
- `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predictLongRunning`
- `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:fetchPredictOperation`
- `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict`
- `/v1/music`
- `/v1/music_generation`
- `/v1/lyrics_generation`
- `/v1/video_generation`
- `/v1/query/video_generation`
- `/v1/files/retrieve`
- `/ent/v2/text2video`
- `/ent/v2/img2video`
- `/ent/v2/reference2video`
- `/ent/v2/tasks/{id}/creations`
- `/ent/v2/tasks/{id}/cancel`
- `/api/v1/generate`
- `/api/v1/generate/record-info`
- `/api/v1/lyrics`
- `/api/v1/lyrics/record-info`
- `/v1/assistants`
- `/v1/threads`
- `/v1/conversations`
- `/v1/vector_stores`
- `/v1/batches`
- `/v1/fine_tuning/jobs`
- `/v1/webhooks`
- `/v1/evals`
- `/v1/videos`

The `audio` family is currently published as the shared `audio.openai` mirror family on `/v1/audio/*`. The public contract stays on the current shared audio surface and does not introduce wrapper prefixes such as `/audio/openai/*`.

The `music` family is currently published as the shared `music.openai` mirror family on `/v1/music*`. It remains resource-oriented instead of binding the public contract to one provider transport, so routing, billing, and plugin adapters stay aligned with the same capability-first gateway model used by images and videos.

For images, the gateway currently publishes the shared `images.openai` mirror family on `/v1/images/*`, the provider-specific `images.kling` and `images.aliyun` mirror tags on the shared official DashScope `/api/v1/services/aigc/image-generation/generation` and `/api/v1/tasks/{task_id}` paths, and the provider-specific `images.volcengine` mirror family on Volcengine Ark's official `/api/v3/images/generations` path. Nano Banana stays on Google's official Gemini `/v1beta/models/{model}:generateContent` protocol and is therefore covered by `code.gemini` instead of a separate `images.nanobanana` mirror family. `images.midjourney` remains unpublished because Midjourney does not provide an official API surface that can be mirrored by switching only `base_url`.

For video, the gateway currently publishes the shared `video.openai` mirror family on `/v1/videos*`, including Sora 2 and Sora 2 Pro because OpenAI already defines that official transport, the provider-specific `video.kling` and `video.aliyun` mirror families on the shared official DashScope `/api/v1/services/aigc/video-generation/video-synthesis` and `/api/v1/tasks/{task_id}` paths, the provider-specific `video.google-veo` mirror family on Google Vertex AI's official `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predictLongRunning` and `:fetchPredictOperation` paths, including Veo 3-class models selected through `{model}`, the provider-specific `video.minimax` mirror family on MiniMax's official `/v1/video_generation`, `/v1/query/video_generation`, and `/v1/files/retrieve` paths, the provider-specific `video.vidu` mirror family on Vidu's official `/ent/v2/text2video`, `/ent/v2/img2video`, `/ent/v2/reference2video`, `/ent/v2/tasks/{id}/creations`, and `/ent/v2/tasks/{id}/cancel` paths, and the provider-specific `video.volcengine` mirror family on Volcengine's official `/api/v1/contents/generations/tasks` and `/api/v1/contents/generations/tasks/{id}` paths. The gateway therefore does not publish a separate `video.sora` OpenAPI tag or wrapper route family.

For music, the gateway currently publishes the shared `music.openai` mirror family on `/v1/music*`, the provider-specific `music.google` mirror family on Google Vertex AI's official `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict` path, the provider-specific `music.minimax` mirror family on MiniMax's official `/v1/music_generation` and `/v1/lyrics_generation` paths, and the provider-specific `music.suno` mirror family on Suno's official `/api/v1/generate*` and `/api/v1/lyrics*` paths.

The control plane also exposes:

- `/admin/*`
- `/portal/*`

## Agent Client Compatibility

OpenAI-compatible tool access is the default path for:

- Codex
- OpenCode
- OpenClaw provider manifests
- general OpenAI SDK and CLI clients using `/v1/models`, `/v1/chat/completions`, and `/v1/responses`

Two translated mirror protocol families are first-class public contracts:

- Claude mirror protocol for Claude Code and other Anthropic clients on `POST /v1/messages` and `POST /v1/messages/count_tokens`
- Gemini mirror protocol for Gemini CLI gateway mode and image-capable Gemini clients such as Nano Banana on `POST /v1beta/models/{model}:generateContent`, `POST /v1beta/models/{model}:streamGenerateContent?alt=sse`, and `POST /v1beta/models/{model}:countTokens`

These routes do not bypass SDKWork routing. They translate into the same execution path used by the OpenAI-compatible gateway, so provider selection, project routing preferences, quota enforcement, billing, and usage recording stay consistent across all three protocol families.

Stateful deployments accept the official protocol auth inputs in addition to bearer tokens:

- Claude surface: `Authorization: Bearer ...` or `x-api-key`
- Gemini surface: `Authorization: Bearer ...`, `x-goog-api-key`, or `?key=...`

Compatibility-specific transport details that are now preserved explicitly:

- Claude relay keeps `anthropic-version` and `anthropic-beta` when the request fans out to upstream runtime adapters
- Gemini CLI quick setup is aligned with the current official client path that uses `GOOGLE_GEMINI_BASE_URL` and `GEMINI_API_KEY_AUTH_MECHANISM=bearer`

Inside `sdkwork-router-portal`, the `Gateway` workspace route now surfaces this compatibility story directly in-product so operators can see the relationship between client setup, deployment mode, and billing posture without switching to the docs first.

## How To Read Compatibility

- use the API reference pages to understand public mirror protocol families, base paths, and auth
- use this compatibility view to understand execution semantics
- use the full matrix when you need route-family-level truth across stateful and stateless modes

Primary entry points:

- [API Reference Overview](/api-reference/overview)
- [Gateway API Reference](/api-reference/gateway-api)
- [Admin API Reference](/api-reference/admin-api)
- [Portal API Reference](/api-reference/portal-api)

## Detailed References

Read the full data-plane and control-plane matrix here:

- [Full Compatibility Matrix](/api/compatibility-matrix)
