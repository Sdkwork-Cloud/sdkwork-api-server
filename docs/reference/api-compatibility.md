# API Compatibility

SDKWork tracks compatibility with five execution-truth labels:

- `native`
- `relay`
- `translated`
- `emulated`
- `unsupported`

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
- `/v1/assistants`
- `/v1/threads`
- `/v1/conversations`
- `/v1/vector_stores`
- `/v1/batches`
- `/v1/fine_tuning/jobs`
- `/v1/webhooks`
- `/v1/evals`
- `/v1/videos`

The control plane also exposes:

- `/admin/*`
- `/portal/*`

## Agent Client Compatibility

OpenAI-compatible tool access is the default path for:

- Codex
- OpenCode
- OpenClaw provider manifests
- general OpenAI SDK and CLI clients using `/v1/models`, `/v1/chat/completions`, and `/v1/responses`

Two translated gateway surfaces are now first-class:

- Anthropic Messages compatibility for Claude Code and other Anthropic clients on `POST /v1/messages` and `POST /v1/messages/count_tokens`
- Gemini Generative Language compatibility for Gemini CLI gateway mode on `POST /v1beta/models/{model}:generateContent`, `POST /v1beta/models/{model}:streamGenerateContent?alt=sse`, and `POST /v1beta/models/{model}:countTokens`

These routes do not bypass SDKWork routing. They translate into the same execution path used by the OpenAI-compatible gateway, so provider selection, project routing preferences, quota enforcement, billing, and usage recording stay consistent across all three protocol families.

Stateful deployments accept the compatibility-native auth inputs in addition to bearer tokens:

- Anthropic surface: `Authorization: Bearer ...` or `x-api-key`
- Gemini surface: `Authorization: Bearer ...`, `x-goog-api-key`, or `?key=...`

Compatibility-specific transport details that are now preserved explicitly:

- Anthropic Messages relay keeps `anthropic-version` and `anthropic-beta` when the request fans out to upstream runtime adapters
- Gemini CLI quick setup is aligned with the current official client path that uses `GOOGLE_GEMINI_BASE_URL` and `GEMINI_API_KEY_AUTH_MECHANISM=bearer`

Inside `sdkwork-router-portal`, the `Gateway` workspace route now surfaces this compatibility story directly in-product so operators can see the relationship between client setup, deployment mode, and billing posture without switching to the docs first.

## How To Read Compatibility

- use the API reference pages to understand ownership, base paths, and auth
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
