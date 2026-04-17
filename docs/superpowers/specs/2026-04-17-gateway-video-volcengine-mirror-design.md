# Gateway Video Volcengine Mirror Design

## Superseded Status

This slice is implemented, but one detail is outdated: `images.nanobanana` is no longer treated as a reserved future image family, and `video.sora` is not a future wrapper family for public governance. Nano Banana stays under `code.gemini`, while Sora 2 and Sora 2 Pro stay under the shared `video.openai` contract.

## Goal

Publish `video.volcengine` as an active provider-specific mirror family on Volcengine's official video-generation transport without adding wrapper routes.

## Scope

This slice activates the official Volcengine video task protocol exposed on:

- `POST /api/v1/contents/generations/tasks`
- `GET /api/v1/contents/generations/tasks/{id}`

The gateway should remain a strict mirror router: existing Volcengine clients only need to switch `base_url`.

## Design

`video.volcengine` is a provider-specific top-level family because OpenAI does not define an equivalent video-task protocol. The gateway should therefore publish the official Volcengine paths exactly as-is and keep request and response schemas as passthrough `serde_json::Value`.

Stateful behavior:

- create requests select a `videos` provider by request `model` and mirror identity `volcengine`
- task queries resolve provider ownership from the latest billing event whose `reference_id` matches the task `id`
- successful create and get calls record usage and billing with the task `id` as `reference_id`

Stateless behavior:

- relay the request only when `upstream.mirror_protocol_identity() == "volcengine"`
- preserve the official path and JSON payload unchanged

## Route Keys

- `video.volcengine.tasks.create`
- `video.volcengine.tasks.get`

## OpenAPI and Docs

OpenAPI should publish a new `video.volcengine` tag and document the two official paths under that tag. Public docs should move `video.volcengine` from reserved-only status to active, keep `images.midjourney` unpublished, document that Nano Banana belongs to `code.gemini`, and treat Sora 2 plus Sora 2 Pro as part of the shared `video.openai` `/v1/videos*` contract instead of a separate `video.sora` family.

## Deferred

This slice does not invent any non-official Volcengine wrapper routes and does not guess at additional endpoints beyond the official create/get task protocol validated for the current mirror surface.
